use crate::{libs::notifier::Notifier, perf::PerfData};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    thread::park_timeout,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Command {
    #[default]
    None,
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
    pub static ref SCH1HZ: Notifier = Notifier::new();
    pub static ref SCH5HZ: Notifier = Notifier::new();
    pub static ref SCH10HZ: Notifier = Notifier::new();
    pub static ref SCH25HZ: Notifier = Notifier::new();
    pub static ref SCH50HZ: Notifier = Notifier::new();
    pub static ref SCH100HZ: Notifier = Notifier::new();
}

pub fn start() {
    let interval = Duration::from_millis(10);
    let mut next_time = Instant::now() + interval;
    let mut data = Data::default();

    loop {
        park_timeout(next_time.duration_since(Instant::now()));
        data.out.perf.enter();

        SCH100HZ.notify();
        if data.out.counter % 2 == 0 {
            SCH50HZ.notify();
        }
        if data.out.counter % 4 == 0 {
            SCH25HZ.notify();
        }
        if data.out.counter % 10 == 0 {
            SCH10HZ.notify();
        }
        if data.out.counter % 20 == 0 {
            SCH5HZ.notify();
        }
        if data.out.counter % 100 == 0 {
            SCH1HZ.notify();
        }

        data.out.counter += 1;
        next_time += interval;
        data.out.perf.exit();
    }
}
