use crate::{libs::msg_queue::MsgOut, perf::PerfData, sch, to};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

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
    pub static ref OUT_DATA: MsgOut<Out, 1> = MsgOut::new();
}

pub fn start() {
    let mut data = Data::default();
    let notifier = sch::SCH1HZ.subscribe();
    loop {
        notifier.wait();
        data.out.perf.enter();
        to::SEND_QUEUE.push(crate::SbMsg::ExampleOut(data.out));
        data.out.counter += 1;
        println!("data {:?}", data.out);
        data.out.perf.exit();
    }
}
