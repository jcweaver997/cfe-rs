pub mod msg;
pub mod perf;
pub mod udp;
use std::{
    collections::HashSet,
    os::fd::RawFd,
    time::{Duration, Instant},
};

use log::*;
use msg::{SbMsg, SbSubRes, SbSubReq};

pub trait TCfeConnection {
    fn send_message(&mut self, msg: &[u8]);
    fn recv_message(&mut self) -> Vec<u8>;
    fn get_fd(&self) -> RawFd;
}

pub struct CfeConnection {
    con: Box<dyn TCfeConnection>,
    verified_subs: HashSet<u64>,
    last_requested: Instant,
    posts: HashSet<u64>,
}

impl CfeConnection {
    pub fn send_message(&mut self, msg: &[u8], desc: u64) {
        let desc_req = unsafe { std::mem::transmute(std::mem::discriminant(&SbMsg::SbSubReq(SbSubReq::default()))) };
        let desc_res = unsafe { std::mem::transmute(std::mem::discriminant(&SbMsg::SbSubRes(SbSubRes::default()))) };
        if self.posts.contains(&desc) || desc == desc_req || desc == desc_res {
            self.con.send_message(msg);
        }
    }

    pub fn recv_message(&mut self) -> Vec<u8> {
        return self.con.recv_message();
    }

    pub fn verify_subs(&mut self, requested_subs: &HashSet<u64>) {
        if *requested_subs == self.verified_subs {
            return;
        }
        let now = Instant::now();
        info!("verf {}", now.duration_since(self.last_requested).as_secs_f32());
        if now.duration_since(self.last_requested).as_secs_f32() > 5.0 {
            self.last_requested = now;
            let msg = msg::SbMsg::SbSubReq(msg::SbSubReq {
                subs: requested_subs.clone(),
            });
            if let Ok(v) = bincode::serialize::<msg::SbMsg>(&msg) {
                let desc: u64 = unsafe { std::mem::transmute(std::mem::discriminant(&msg)) };
                self.send_message(&v, desc);
            } else {
                error!("failed to serialize msg {:?}", msg);
            }
        }
    }
}

pub struct Cfe {
    connections: Vec<CfeConnection>,
    requested_subs: HashSet<u64>,
    epoll_fd: RawFd,
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
    pub fn init_cfe() -> Cfe {
        let cf = Cfe {
            connections: Vec::new(),
            requested_subs: HashSet::new(),
            epoll_fd: epoll::create(false).expect("failed to create epoll fd"),
        };
        info!("CFE Initialized");
        return cf;
    }

    pub fn add_connection(&mut self, connection: Box<dyn TCfeConnection>) {
        let mut con = CfeConnection {
            con: connection,
            verified_subs: HashSet::new(),
            last_requested: Instant::now() - Duration::from_secs(10000),
            posts: HashSet::new(),
        };
        epoll::ctl(
            self.epoll_fd,
            epoll::ControlOptions::EPOLL_CTL_ADD,
            con.con.get_fd(),
            epoll::Event::new(epoll::Events::EPOLLIN, 0),
        ).expect("failed to add fd to epoll");

        con.verify_subs(&self.requested_subs);
        self.connections.push(con);
    }

    pub fn send_message(&mut self, msg: msg::SbMsg) {
        if let Ok(v) = bincode::serialize::<msg::SbMsg>(&msg) {
            let desc: u64 = unsafe { std::mem::transmute(std::mem::discriminant(&msg)) };
            for con in &mut self.connections {
                con.send_message(&v, desc);
            }
        } else {
            error!("failed to serialize msg {:?}", msg);
            return;
        }
    }

    pub fn recv_message(&mut self, blocking: bool) -> Option<SbMsg> {
        self.poll();

        if blocking {
            let mut events = Vec::new();
            events.resize(self.connections.len(), unsafe{std::mem::zeroed::<epoll::Event>()});
            epoll::wait(self.epoll_fd, 1000, &mut events).ok();
        }

        for con in &mut self.connections {
            let v = con.recv_message();
            if v.len() > 0 {
                let msg = bincode::deserialize(&v).ok();
                match msg {
                    Some(SbMsg::SbSubRes(res)) => {
                        con.verified_subs = res.subs;
                    },
                    Some(SbMsg::SbSubReq(req)) => {
                        con.posts = req.subs;
                    }
                    _ => {
                        return msg;
                    }
                }
            }
        }

        return None;
    }

    pub fn poll(&mut self) {
        for con in &mut self.connections {
            con.verify_subs(&self.requested_subs);
        }
    }

    pub fn subscribe(&mut self, t: SbMsg) {
        let i: u64 = unsafe { std::mem::transmute(std::mem::discriminant(&t)) };
        trace!("Adding {} to requested subscriptions", i);
        self.requested_subs.insert(i);
    }
}
