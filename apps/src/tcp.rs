use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread::{spawn, JoinHandle},
};

use crate::{
    libs::msg_queue::{MsgOut, MsgQueue},
    perf::PerfData,
    sch, to,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use ringbuf::Rb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Command {
    #[default]
    None,
    Enable,
    Disable,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Out {
    pub perf: PerfData,
    pub counter: u32,
    pub rx_bytes: u32,
    pub tx_bytes: u32,
}

#[derive(Debug, Default)]
struct Data {
    out: Out,
}

pub const UNIT_SIZE: usize = 4096;

lazy_static! {
    pub static ref CMD_QUEUE: MsgQueue<Command, 10> = MsgQueue::new();
    pub static ref TX_BUF: MsgQueue<u8, { UNIT_SIZE }> = MsgQueue::new();
    pub static ref RX_BUF: MsgOut<u8, { UNIT_SIZE }> = MsgOut::new();
}

pub struct Init {
    pub bind_addr: String,
    pub peer_addr: String,
    pub default_enabled: bool,
}

pub fn start(init: Init) {
    let notifier = sch::SCH25HZ.subscribe();

    let mut data = Data::default();
    let mut read_buf = [0u8; UNIT_SIZE];

    let mut socket = None;
    let mut listen_peer = None;

    let mut connect: Option<JoinHandle<()>> = None;
    let mut enabled = init.default_enabled;

    let send_peer: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
    loop {
        notifier.wait();
        data.out.perf.enter();

        for cmd in CMD_QUEUE.lock().pop_iter() {
            info!("got cmd {:?}", cmd);
            match cmd {
                Command::Enable => enabled = true,
                Command::Disable => enabled = false,
                Command::None => {}
            }
        }

        if enabled {
            if socket.is_none() {
                socket = TcpListener::bind(init.bind_addr.clone()).ok();
                if let Some(s) = &socket {
                    s.set_nonblocking(true).unwrap();
                }
            }
            if let Some(s) = &socket {
                if let Some(c) = s.accept().ok() {
                    c.0.set_nonblocking(true).unwrap();
                    listen_peer = Some(c.0);
                }
            }
            {
                let mut buf = TX_BUF.lock();
                let mut peer = send_peer.lock();
                if let Some(s) = &mut *peer {
                    if buf.len() > 0 {
                        match s.write(&buf.pop_iter().take(UNIT_SIZE).collect::<Vec<u8>>()) {
                            Err(e) => {
                                error!("{e}");
                                peer.take().unwrap();
                            }
                            Ok(s) => data.out.tx_bytes = s as u32,
                        }
                    }
                }
            }
            let mut connect_running = false;
            if let Some(c) = &connect {
                connect_running = !c.is_finished();
            }
            if send_peer.lock().is_none() && !connect_running {
                let sm = send_peer.clone();
                let addr = init.peer_addr.clone();
                connect = Some(spawn(move || {
                    if let Ok(stream) = TcpStream::connect(addr) {
                        stream.set_nonblocking(true).unwrap();
                        *sm.lock() = Some(stream);
                    }
                }));
            }
            if let Some(c) = &mut listen_peer {
                match c.read(&mut read_buf) {
                    Ok(s) => {
                        data.out.rx_bytes += s as u32;
                        RX_BUF.send_slice(&read_buf[0..s]);
                    }
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            error!("{e}");
                            drop(listen_peer.take().unwrap());
                        }
                    }
                }
            }
        }
        to::SEND_QUEUE.push(crate::SbMsg::TcpOut(data.out));
        data.out.counter += 1;
        data.out.perf.exit();
    }
}
