use crate::SbMsg;
use crate::{
    libs::msg_queue::{MsgOut, MsgQueue},
    perf::PerfData,
    sch,
    tcp::TX_BUF,
};
use lazy_static::lazy_static;
use log::*;
use ringbuf::Rb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Command {
    #[default]
    None,
    Enable,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Out {
    pub perf: PerfData,
    pub counter: u32,
}

#[derive(Debug, Default)]
struct Data {
    out: Out,
}

lazy_static! {
    pub static ref CMD_QUEUE: MsgQueue<Command, 10> = MsgQueue::new();
    pub static ref SEND_QUEUE: MsgQueue<SbMsg, 50> = MsgQueue::new();
    pub static ref OUT_DATA: MsgOut<Out, 1> = MsgOut::new();
}

pub fn start() {
    let notifier = sch::SCH25HZ.subscribe();
    let mut data = Data::default();

    loop {
        notifier.wait();
        data.out.perf.enter();
        for cmd in CMD_QUEUE.lock().pop_iter() {
            handle_cmd(&mut data, cmd);
        }

        OUT_DATA.send(data.out);
        SEND_QUEUE.push(SbMsg::ToOut(data.out));
        data.out.counter += 1;

        let mut send_counter = 0;
        for msg in SEND_QUEUE.lock().pop_iter() {
            send_counter += 1;
            if let Ok(v) = bincode::serialize::<SbMsg>(&msg) {
                let mut txbuf = TX_BUF.lock();
                if txbuf.free_len().clone() >= v.len() + 4 {
                    txbuf.push_slice(&(v.len() as u32).to_be_bytes());
                    txbuf.push_slice(&v);
                }
            }
            if send_counter > 10 {
                break;
            }
        }
        data.out.perf.exit();
    }
}

fn handle_cmd(_data: &mut Data, cmd: Command) {
    match cmd {
        Command::None => info!("got none"),
        Command::Enable => info!("got enable"),
    }
}
