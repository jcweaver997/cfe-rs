use std::{
    fs,
    io::ErrorKind,
    os::unix::net::UnixDatagram,
    sync::{Arc, Mutex, Condvar},
    thread::spawn,
};

use log::*;
use queues::{Buffer, IsQueue};

use crate::{msg::SbMsg, TCfeConnection};

pub struct SbUnix {
    u: UnixDatagram,
    target: String,
}

impl SbUnix {
    pub fn new(bind_path: &str, target_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if std::path::Path::new(bind_path).exists() {
            fs::remove_file(bind_path)?;
        }
        let u = UnixDatagram::bind(bind_path)?;
        return Ok(Self {
            u,
            target: target_path.to_string(),
        });
    }
}

impl SbUnix {
    fn recv_thread(u: UnixDatagram, msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>, token: usize) {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        loop {
            match u.recv(&mut packet) {
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        error!("received unix recv error {}", e);
                    }
                }
                Ok(s) => {
                    if let Ok(msg) = bincode::deserialize(&packet[0..s]) {
                        if let Err(_) = msg_queue.0
                            .lock()
                            .expect("failed to acquire mutex")
                            .add((msg, token))
                        {
                            error!("failed to push to ring buffer");
                        }
                        msg_queue.1.notify_one();
                    } else {
                        error!("failed to deserialize msg");
                    }
                }
            }
        }
    }
}

impl TCfeConnection for SbUnix {
    fn send_message(&mut self, msg: &SbMsg) {
        if let Ok(msg) = bincode::serialize(msg) {
            if msg.len() > 65507 {
                error!("msg too big for unix {}", msg.len());
                return;
            }
            trace!("sending unix msg len {}", msg.len());
            if let Err(e) = self.u.send_to(&msg, &self.target) {
                if e.kind() != ErrorKind::NotFound && e.kind() != ErrorKind::ConnectionRefused {
                    error!("received unix send error {:?}", e.kind());
                }
            }
        } else {
            error!("failed to serialize msg");
        }
    }

    fn start_recv(&mut self, msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>, token: usize) {
        let u = self.u.try_clone().expect("failed to clone unix socket");
        spawn(move || Self::recv_thread(u, msg_queue, token));
    }
}
