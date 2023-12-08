pub mod msg;
pub mod perf;
pub mod sbn;

use std::{
    collections::HashSet,
    os::fd::RawFd,
    time::{Duration, Instant},
};

use log::*;
use msg::{AppName, Computer, SbMsg, SbMsgData, SbSubReq};

const HEARTBEAT_DELAY: f32 = 1.0;
const MAX_HEARTBEAT_MISS: u32 = 3;

pub trait TCfeConnection {
    fn send_message(&mut self, msg: &[u8]);
    fn recv_message(&mut self) -> Vec<u8>;
    fn get_fd(&mut self) -> RawFd;
}

pub struct CfeConnection {
    con: Box<dyn TCfeConnection>,
    requested_subs: HashSet<(u64, Computer)>,
    posts: HashSet<(u64, Computer)>,
    last_heartbeat_sent: Instant,
    last_heartbeat_received: Instant,
    pub connected: bool,
    pub computer: msg::Computer,
    pub app_name: msg::AppName,
    pub send_seq: u16,
    pub recv_seq: u16,
    pub extra_posts: HashSet<(u64, Computer)>
}

impl CfeConnection {
    pub fn new(connection: Box<dyn TCfeConnection>) -> Self {
        return CfeConnection {
            con: connection,
            requested_subs: HashSet::new(),
            posts: HashSet::new(),
            last_heartbeat_sent: Instant::now() - Duration::from_secs(10000),
            last_heartbeat_received: Instant::now() - Duration::from_secs(10000),
            computer: Computer::None,
            connected: false,
            app_name: msg::AppName::None,
            send_seq: 0,
            recv_seq: 0,
            extra_posts: HashSet::new()
        };
    }

    pub fn send_message(&mut self, msg: &[u8], desc: u64, computer: Computer) {
        let desc_req = SbMsgData::SbSubReq(SbSubReq::default()).get_id();
        let desc_heartbeat = SbMsgData::None.get_id();
        if self.posts.contains(&(desc, computer)) || desc == desc_req || desc == desc_heartbeat {
            self.con.send_message(msg);
            self.send_seq += 1;
        }
    }

    pub fn recv_message(&mut self) -> Vec<u8> {
        return self.con.recv_message();
    }

    pub fn subscribe(&mut self, t: SbMsgData, computer: Computer) {
        let i: u64 = t.get_id();
        trace!("Adding {} to requested subscriptions", i);
        self.requested_subs.insert((i, computer));
    }

    pub fn send_subs(
        &mut self,
        computer: Computer,
        app_name: AppName,
    ) {
        let mut subs = self.requested_subs.clone();
        subs.extend(self.extra_posts.iter());
        let data = msg::SbMsgData::SbSubReq(msg::SbSubReq { subs });
        let msg = SbMsg {
            data,
            computer,
            app_name,
            sequence: self.send_seq,
        };
        if let Ok(v) = msg.serialize() {
            let desc: u64 = msg.data.get_id();
            self.send_message(&v, desc, computer);
        } else {
            error!("failed to serialize msg {:?}", msg);
        }
    }

    pub fn heartbeat(&mut self, computer: Computer, app_name: AppName) {
        let now = Instant::now();
        if now.duration_since(self.last_heartbeat_sent).as_secs_f32() >= HEARTBEAT_DELAY {
            self.last_heartbeat_sent = now;
            let msg = SbMsg {
                data: SbMsgData::None,
                computer,
                app_name,
                sequence: self.send_seq,
            };
            self.send_message(
                &msg.serialize().expect("failed to serialize heartbeat"),
                SbMsgData::None.get_id(),
                computer,
            );
        }
        if now
            .duration_since(self.last_heartbeat_received)
            .as_secs_f32()
            >= HEARTBEAT_DELAY * MAX_HEARTBEAT_MISS as f32
        {
            if self.connected {
                self.connected = false;
                warn!(
                    "heartbeat lost from {:?}.{:?}",
                    self.computer, self.app_name
                );
            }
        }
    }

    pub fn heartbeat_received(&mut self, msg: SbMsg) {
        self.last_heartbeat_received = Instant::now();
        self.computer = msg.computer;
        self.app_name = msg.app_name;
        if !self.connected {
            self.connected = true;
            info!(
                "heartbeat started from {:?}.{:?}",
                self.computer, self.app_name
            );
        }
    }
}

pub struct Cfe {
    pub connections: Vec<CfeConnection>,
    poll: RawFd,
    poll_events: [epoll::Event; 64],
    pub computer: msg::Computer,
    pub app_name: msg::AppName,
    pub relay: bool,
}

pub trait SbApp {
    fn init(&mut self);
    fn start(&mut self);

    fn run(&mut self) {
        self.init();
        self.start();
    }
}

impl Cfe {
    pub fn init_cfe(computer: msg::Computer, app_name: msg::AppName) -> Cfe {
        let cf = Cfe {
            connections: Vec::new(),
            poll: epoll::create(false).expect("failed to create epoll"),
            poll_events: [epoll::Event { data: 0, events: 0 }; 64],
            computer,
            app_name,
            relay: false,
        };
        info!("CFE Initialized");
        return cf;
    }

    pub fn add_connection(&mut self, mut connection: CfeConnection) {
        epoll::ctl(
            self.poll,
            epoll::ControlOptions::EPOLL_CTL_ADD,
            connection.con.get_fd(),
            epoll::Event {
                events: epoll::Events::EPOLLIN.bits(),
                data: self.connections.len() as u64,
            },
        )
        .expect("Failed to add connection to epoll");
        connection.send_subs(self.computer, self.app_name);
        self.connections.push(connection);
        self.poll();
    }

    pub fn send_message(&mut self, data: msg::SbMsgData) {
        for con in &mut self.connections {
            let msg = SbMsg {
                data: data.clone(),
                computer: self.computer,
                app_name: self.app_name,
                sequence: con.send_seq,
            };
            let desc: u64 = msg.data.get_id();
            if let Ok(v) = msg.serialize() {
                con.send_message(&v, desc, self.computer);
            } else {
                error!("failed to serialize msg {:?}", msg);
                return;
            }
        }
    }

    pub fn recv_message(&mut self, blocking: bool) -> Option<SbMsg> {
        self.poll();
        let mut con_i = 0;
        let v = if blocking {
            match epoll::wait(
                self.poll,
                (HEARTBEAT_DELAY * 1000.0) as i32,
                &mut self.poll_events,
            ) {
                Ok(s) => {
                    if s > 0 {
                        // self.connections[self.poll_events[0].data as usize].recv_message()

                        let mut v = Vec::new();
                        for i in 0..self.connections.len() {
                            v = self.connections[i].recv_message();
                            if v.len() > 0 {
                                con_i = i;
                                break;
                            }
                        }
                        v
                    } else {
                        Vec::new()
                    }
                }
                Err(e) => {
                    error!("poll error {e}");
                    Vec::new()
                }
            }
        } else {
            let mut v = Vec::new();
            for i in 0..self.connections.len() {
                v = self.connections[i].recv_message();
                if v.len() > 0 {
                    con_i = i;
                    break;
                }
            }
            v
        };

        if v.len() > 0 {
            match SbMsg::deserialize(&v) {
                Ok(mut m) => {
                    if self.connections[con_i].recv_seq + 1 != m.sequence {
                        if m.sequence == 0 {
                            self.connections[con_i].send_subs(self.computer, self.app_name);
                        }else{
                            error!(
                                "got mismatched sequence number from {:?}.{:?}, expected {} got {}",
                                self.connections[con_i].computer,
                                self.connections[con_i].app_name,
                                self.connections[con_i].recv_seq + 1,
                                m.sequence
                            );
                        }

                    }
                    self.connections[con_i].recv_seq = m.sequence;


                    match m.data {
                        SbMsgData::SbSubReq(req) => {
                            self.connections[con_i].posts = req.subs;
                        }
                        SbMsgData::None => {
                            if !self.connections[con_i].connected {
                                self.connections[con_i].send_subs(self.computer, self.app_name);
                            }
                            self.connections[con_i].heartbeat_received(m);
                        }
                        _ => {
                            if self.relay {
                                let mid = m.data.get_id();

                                for con_forward in 0..self.connections.len() {
                                    if con_forward == con_i {
                                        continue;
                                    }
                                    m.sequence = self.connections[con_forward].send_seq;
                                    if let Ok(forward_bytes) = m.serialize() {
                                        self.connections[con_forward].send_message(
                                            &forward_bytes,
                                            mid,
                                            m.computer,
                                        );
                                    } else {
                                        error!("failed to serialize msg {:?}", m);
                                    }

                                }
                            }
                            return Some(m);
                        }
                    }
                }
                Err(e) => {
                    error!("failed to deserialize packet {}", e);
                    return None;
                }
            }
        }

        return None;
    }

    pub fn poll(&mut self) {
        let mut all_posts: HashSet<(u64, Computer)> = HashSet::new();
        if self.relay {
            for con in &mut self.connections {
                all_posts.extend(con.posts.iter());
            }
            for con in &mut self.connections {
                con.extra_posts = all_posts.clone();
            }
        }
        for con in &mut self.connections {
            con.heartbeat(self.computer, self.app_name);
        }
    }
}
