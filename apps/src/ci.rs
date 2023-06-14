use crate::libs::msg_queue::MsgOut;
use crate::{libs::msg_queue::MsgQueue, perf::PerfData, sch};
use crate::{tcp, SbMsg};
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
    pub static ref IN_BUF: MsgQueue<u8, { tcp::UNIT_SIZE }> = MsgQueue::new();
    pub static ref MSGS: MsgOut<SbMsg, 10> = MsgOut::new();
}

pub fn start() {
    let notifier = sch::SCH50HZ.subscribe();
    let mut data = Data::default();
    tcp::RX_BUF.subscribe(&IN_BUF);

    loop {
        notifier.wait();
        data.out.perf.enter();

        let mut inbuf = IN_BUF.lock();
        for _ in 0..20 {
            if inbuf.len() > 4 {
                let packet_length = u32::from_be_bytes(
                    inbuf
                        .iter()
                        .take(4)
                        .copied()
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap(),
                );

                if inbuf.len() >= 4 + packet_length as usize {
                    let packet: Vec<u8> = inbuf
                        .pop_iter()
                        .skip(4)
                        .take(packet_length as usize)
                        .collect();

                    match bincode::deserialize::<SbMsg>(&packet) {
                        Ok(msg) => {
                            MSGS.send(msg);
                        }
                        Err(e) => {
                            info!("{e}");
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // if let Ok(v) = bincode::serialize(&SbMsg::ToOut(data.out)) {
        //     udp::TX_BUF.lock().push_iter(&mut v.into_iter());
        // }

        data.out.perf.exit();
    }
}
