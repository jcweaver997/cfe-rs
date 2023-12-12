use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, sync::{Arc, Mutex, Condvar}, thread::spawn};

use log::*;
use queues::{Buffer, IsQueue};

use crate::{TCfeConnection, msg::SbMsg};

pub struct SbUdpServer {
    remote_addr: Arc<Mutex<Option<SocketAddr>>>,
    udp: UdpSocket,
}

impl SbUdpServer {
    pub fn new(bind_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let udp = UdpSocket::bind(bind_addr)?;
        return Ok(Self {
            remote_addr: Arc::new(Mutex::new(None)),
            udp,
        });
    }

    fn recv_thread(
        u: UdpSocket,
        msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>,
        set_remote_addr: Arc<Mutex<Option<SocketAddr>>>,
        token: usize
    ) {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        loop {
            match u.recv_from(&mut packet) {
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        error!("received udp recv error {}", e);
                    }
                }
                Ok((s,remote_addr)) => {
                    *set_remote_addr.lock().expect("failed to acquire mutex") = Some(remote_addr);
                    if let Ok(msg) = bincode::deserialize(&packet[0..s]) {
                        if let Err(_) = msg_queue.0.lock().expect("failed to acquire mutex").add((msg, token)) {
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

impl TCfeConnection for SbUdpServer {
    fn send_message(&mut self, msg: &SbMsg) {
        let remote_addr = if let Some(remote_addr) = *self.remote_addr.lock().expect("failed to acquire mutex") {remote_addr} else {return;};
        if let Ok(msg) = bincode::serialize(msg) {
            if msg.len() > 65507 {
                error!("msg too big for udp {}", msg.len());
                return;
            }
            trace!("sending udp msg len {}", msg.len());
            if let Err(e) = self.udp.send_to(&msg, remote_addr) {
                if e.kind() != ErrorKind::NotFound && e.kind() != ErrorKind::ConnectionRefused {
                    error!("received udp send error {:?}", e.kind());
                }
            }
        } else {
            error!("failed to serialize msg");
        }
    }

    fn start_recv(
        &mut self,
        msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>,
        token: usize
    ) {
        let u = self.udp.try_clone().expect("failed to clone unix socket");
        let remote_addr = self.remote_addr.clone();
        spawn(move || Self::recv_thread(u, msg_queue, remote_addr, token));
    }
}
