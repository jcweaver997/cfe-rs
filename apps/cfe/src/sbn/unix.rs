use std::{
    io::ErrorKind, fs, os::{unix::net::UnixDatagram, fd::{AsRawFd, RawFd}},
};

use log::*;

use crate::TCfeConnection;

pub struct SbUnix {
    u: UnixDatagram,
    target: String,
}

impl SbUnix {
    pub fn new(bind_path: &str, target_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if std::path::Path::new(bind_path).exists() {
            fs::remove_file(bind_path)?;
        }
        let u = std::os::unix::net::UnixDatagram::bind(bind_path)?;
        u.set_nonblocking(true)?;
        return Ok(Self {
            u,
            target: target_path.to_string(),
        });
    }
}

impl TCfeConnection for SbUnix {
    fn send_message(&mut self, msg: &[u8]) {
        if msg.len() > 65507 {
            error!("msg too big for unix {}", msg.len());
            return;
        }
        trace!("sending unix msg len {}", msg.len());
        if let Err(e) = self.u.send_to(msg, &self.target) {
            if e.kind() != ErrorKind::NotFound && e.kind() != ErrorKind::ConnectionRefused {
                error!("received unix send error {:?}", e.kind());
            }
        }
    }

    fn recv_message(&mut self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.resize(65507, 0);
        match self.u.recv(&mut packet) {
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
        return self.u.as_raw_fd();
    }


    
}
