use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}, os::fd::{RawFd, AsRawFd}};

use log::*;

use crate::TCfeConnection;

pub struct SbUdp {
    remote_addr: SocketAddr,
    udp: UdpSocket,
}

impl SbUdp {
    pub fn new(bind_addr: SocketAddr, remote_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let udp = UdpSocket::bind(bind_addr)?;
        udp.set_nonblocking(true)?;
        return Ok(Self {
            remote_addr: remote_addr,
            udp
        });
    }
}

impl TCfeConnection for SbUdp {
    fn send_message(&mut self, msg: &[u8]) {
        if msg.len() > 65507 {
            error!("msg too big for udp {}", msg.len());
            return;
        }
        trace!("sending udp msg len {}", msg.len());
        
        if let Err(e) = self.udp.send_to(msg, self.remote_addr) {
            error!("received udp send error {}", e);
        }
    }

    fn recv_message(&mut self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        match self.udp.recv(&mut packet) {
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    error!("received udp recv error {}", e);
                }
            }
            Ok(s) => {
                packet.resize(s, 0);
                return packet;
            }
        }
        return Vec::new();
    }

    fn get_fd(&mut self) -> RawFd {
        self.udp.as_raw_fd()
    }
}
