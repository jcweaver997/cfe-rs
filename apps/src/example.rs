use crate::{libs::msg_queue::MsgOut, perf::PerfData, sch};
use lazy_static::lazy_static;
use log::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Out {
    pub perf: PerfData,
}

lazy_static! {
    pub static ref OUT_DATA: MsgOut<Out, 1> = MsgOut::new();
}

pub fn start() {
    let mut out = Out::default();
    let notifier = sch::SCH1HZ.subscribe();
    loop {
        notifier.wait();
        out.perf.enter();

        info!("{}", out.perf);
        out.perf.exit();
    }
}
