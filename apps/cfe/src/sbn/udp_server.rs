use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, os::fd::{RawFd, AsRawFd}};

use log::*;

use crate::TCfeConnection;

pub struct SbUdpServer {
    remote_addr: Option<SocketAddr>,
    udp: UdpSocket,
}

impl SbUdpServer {
    pub fn new(
        bind_addr: SocketAddr
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let udp = UdpSocket::bind(bind_addr)?;
        udp.set_nonblocking(true)?;
        return Ok(Self {
            remote_addr: None,
            udp,
        });
    }
}

impl TCfeConnection for SbUdpServer {
    fn send_message(&mut self, msg: &[u8]) {
        if msg.len() > 65507 {
            error!("msg too big for udp {}", msg.len());
            return;
        }
        trace!("sending udp msg len {}", msg.len());
        if let Some(remote_addr) = self.remote_addr {
            if let Err(e) = self.udp.send_to(msg, remote_addr) {
                error!("received udp send error {}", e);
            }
        }
    }

    fn recv_message(&mut self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        match self.udp.recv_from(&mut packet) {
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    error!("received udp recv error {}", e);
                }
            }
            Ok((s, remote_addr)) => {
                packet.resize(s, 0);
                self.remote_addr = Some(remote_addr);
                return packet;
            }
        }
        return Vec::new();
    }

    fn get_fd(&mut self) -> RawFd {
        return self.udp.as_raw_fd();
    }
}
