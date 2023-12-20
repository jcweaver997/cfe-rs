use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex, Condvar},
    thread::spawn,
};

use log::*;
use queues::{Buffer, IsQueue};

use crate::{msg::SbMsg, TCfeConnection};

pub struct SbUdp {
    remote_addr: SocketAddr,
    udp: UdpSocket,
}

impl SbUdp {
    pub fn new(
        bind_addr: SocketAddr,
        remote_addr: SocketAddr,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let udp = UdpSocket::bind(bind_addr)?;
        return Ok(Self {
            remote_addr: remote_addr,
            udp,
        });
    }

    fn recv_thread(u: UdpSocket, msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>, token: usize) {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        loop {
            match u.recv(&mut packet) {
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock && e.kind() != ErrorKind::ConnectionReset {
                        error!("received udp recv error {} {}", e, e.kind());
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

impl TCfeConnection for SbUdp {
    fn send_message(&mut self, msg: &SbMsg) {
        if let Ok(msg) = bincode::serialize(msg) {
            if msg.len() > 65507 {
                error!("msg too big for udp {}", msg.len());
                return;
            }
            trace!("sending udp msg len {}", msg.len());
            if let Err(e) = self.udp.send_to(&msg, &self.remote_addr) {
                if e.kind() != ErrorKind::NotFound && e.kind() != ErrorKind::ConnectionRefused {
                    error!("received udp send error {:?}", e.kind());
                }
            }
        } else {
            error!("failed to serialize msg");
        }
    }

    fn start_recv(&mut self, msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>, token: usize) {
        let u = self.udp.try_clone().expect("failed to clone unix socket");
        spawn(move || Self::recv_thread(u, msg_queue, token));
    }
}
