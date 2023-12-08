pub mod msg;
pub mod perf;
pub mod sbn;

use std::{
    collections::HashSet,
    os::fd::RawFd,
    time::{Duration, Instant},
};

use chrono::Utc;
use colored::Colorize;
use log::*;
use msg::{Computer, EventSeverity, SbEvent, SbMsg, SbMsgData, SbSubReq};

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
    pub recv_seq: Option<u16>,
}

impl CfeConnection {
    pub fn new(connection: Box<dyn TCfeConnection>) -> Self {
        return CfeConnection {
            con: connection,
            requested_subs: HashSet::new(),
            posts: HashSet::new(),
            last_heartbeat_sent: Instant::now() - Duration::from_secs_f32(HEARTBEAT_DELAY),
            last_heartbeat_received: Instant::now() - Duration::from_secs_f32(HEARTBEAT_DELAY),
            computer: Computer::None,
            connected: false,
            app_name: msg::AppName::None,
            send_seq: 0,
            recv_seq: None
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
        self.requested_subs.insert((i, computer));
    }

}

pub struct Cfe {
    pub connections: Vec<CfeConnection>,
    poll: RawFd,
    poll_events: [epoll::Event; 64],
    pub computer: msg::Computer,
    pub app_name: msg::AppName,
    pub relay: bool,
    pub log_level: EventSeverity,
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
    pub fn init_cfe(
        computer: msg::Computer,
        app_name: msg::AppName,
        log_level: EventSeverity,
    ) -> Cfe {
        let cf = Cfe {
            connections: Vec::new(),
            poll: epoll::create(false).expect("failed to create epoll"),
            poll_events: [epoll::Event { data: 0, events: 0 }; 64],
            computer,
            app_name,
            relay: false,
            log_level,
        };
        simple_log::quick!("info");

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
        self.connections.push(connection);
        self.send_subs(self.connections.len() - 1);
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
                self.log(SbEvent::SerializeError, EventSeverity::Error, &format!("failed to serialize msg {:?}", msg));
                return;
            }
        }
    }

    pub fn send_message_to(&mut self, data: msg::SbMsgData, i: usize) {
        let msg = SbMsg {
            data: data.clone(),
            computer: self.computer,
            app_name: self.app_name,
            sequence: self.connections[i].send_seq,
        };
        let desc: u64 = msg.data.get_id();
        if let Ok(v) = msg.serialize() {
            self.connections[i].send_message(&v, desc, self.computer);
        } else {
            self.log(SbEvent::SerializeError, EventSeverity::Error, &format!("failed to serialize msg {:?}", msg));
            return;
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
                        con_i = self.poll_events[0].data as usize;
                        self.connections[self.poll_events[0].data as usize].recv_message()

                        // let mut v = Vec::new();
                        // for i in 0..self.connections.len() {
                        //     v = self.connections[i].recv_message();
                        //     if v.len() > 0 {
                        //         con_i = i;
                        //         break;
                        //     }
                        // }
                        // v
                    } else {
                        Vec::new()
                    }
                }
                Err(e) => {
                    self.log(SbEvent::PollError, EventSeverity::Error, &format!("poll error {e}"));
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
                    if let Some(seq) = self.connections[con_i].recv_seq {
                        if seq + 1 != m.sequence {
                            if m.sequence == 0 {
                                self.send_subs(con_i);
                            } else {
                                self.log(SbEvent::SequenceCountError, EventSeverity::Error, &format!("got mismatched sequence number from {:?}.{:?}, expected {} got {}",
                                self.connections[con_i].computer,
                                self.connections[con_i].app_name,
                                seq + 1,
                                m.sequence));
                            }
                        }
                    }

                    self.connections[con_i].recv_seq = Some(m.sequence);

                    match m.data {
                        SbMsgData::SbSubReq(req) => {
                            self.connections[con_i].posts = req.subs.clone();
                            if self.relay {
                                for c in 0..self.connections.len() {
                                    if c == con_i {
                                        continue;
                                    }
                                    
                                    self.send_subs(c);
                                }
                            }
                        }
                        SbMsgData::None => {
                            if !self.connections[con_i].connected {
                                self.send_subs(con_i);
                            }
                            self.connections[con_i].last_heartbeat_received = Instant::now();
                            self.connections[con_i].computer = m.computer;
                            self.connections[con_i].app_name = m.app_name;
                            if !self.connections[con_i].connected {
                                self.connections[con_i].connected = true;
                                self.log(SbEvent::HeartBeatStarted((self.connections[con_i].computer, self.connections[con_i].app_name)), EventSeverity::Info, &format!("heartbeat started from {:?}.{:?}",
                                self.connections[con_i].computer, self.connections[con_i].app_name));
                            }
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
                                        self.log(SbEvent::SerializeError, EventSeverity::Error, &format!("failed to serialize msg {:?}", m));
                                    }
                                }
                            }
                            return Some(m);
                        }
                    }
                }
                Err(e) => {
                    self.log(SbEvent::DeserializeError, EventSeverity::Error, &format!("failed to deserialize packet {}", e));
                    return None;
                }
            }
        }

        return None;
    }

    pub fn poll(&mut self) {
        self.heartbeat();
    }

    pub fn heartbeat(&mut self) {
        let now = Instant::now();
        for i in 0..self.connections.len() {
            if now
                .duration_since(self.connections[i].last_heartbeat_sent)
                .as_secs_f32()
                >= HEARTBEAT_DELAY
            {
                self.connections[i].last_heartbeat_sent = now;
                self.send_message_to(SbMsgData::None, i);
            }
            if now
                .duration_since(self.connections[i].last_heartbeat_received)
                .as_secs_f32()
                >= HEARTBEAT_DELAY * MAX_HEARTBEAT_MISS as f32
            {
                if self.connections[i].connected {
                    self.connections[i].connected = false;
                    self.log(SbEvent::HeartBeatStopped((self.connections[i].computer, self.connections[i].app_name)), EventSeverity::Warn, &format!("heartbeat lost from {:?}.{:?}",
                    self.computer, self.app_name));
                }
            }
        }
    }

    pub fn send_subs(&mut self, i: usize) {
        let mut subs = self.connections[i].requested_subs.clone();
        if self.relay {
            for con_i in 0..self.connections.len() {
                subs.extend(self.connections[con_i].posts.clone());
            }
        }
        let data = msg::SbMsgData::SbSubReq(msg::SbSubReq { subs });
        self.send_message_to(data, i);
    }

    pub fn log(&mut self, event: SbEvent, severity: EventSeverity, text: &str) {
        let time = Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();
        match severity {
            EventSeverity::Trace if self.log_level <= EventSeverity::Trace => {
                self.send_message(SbMsgData::TraceMsg(event.clone()));
                println!("{} [{}] {}", time, "TRACE".bright_black(), text);
            }
            EventSeverity::Debug if self.log_level <= EventSeverity::Debug => {
                self.send_message(SbMsgData::DebugMsg(event.clone()));
                println!("{} [{}] {}", time, "DEBUG".bright_cyan(), text);
            }
            EventSeverity::Info if self.log_level <= EventSeverity::Info => {
                self.send_message(SbMsgData::InfoMsg(event.clone()));
                println!("{} [{}] {}", time, "INFO".blue(), text);
            }
            EventSeverity::Warn if self.log_level <= EventSeverity::Warn => {
                self.send_message(SbMsgData::WarnMsg(event.clone()));
                println!("{} [{}] {}", time, "WARN".yellow(), text);
            }
            EventSeverity::Error if self.log_level <= EventSeverity::Error => {
                self.send_message(SbMsgData::ErrorMsg(event.clone()));
                println!("{} [{}] {}", time, "ERROR".red(), text);
            }
            _ => {}
        }
    }
}
