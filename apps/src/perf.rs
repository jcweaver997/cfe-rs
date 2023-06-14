use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Instant};

fn now() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerfData {
    #[serde(skip)]
    #[serde(default = "now")]
    last_rate_time: Instant,
    #[serde(skip)]
    #[serde(default = "now")]
    start_time: Instant,
    #[serde(skip)]
    #[serde(default = "now")]
    beginning: Instant,
    pub time_tag: f32,
    pub last_rate: f32,
    pub last_duration: f32,
}

impl PerfData {
    pub fn enter(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_rate = now.duration_since(self.last_rate_time).as_secs_f32();
        self.last_rate_time = now;
        self.time_tag = self.beginning.elapsed().as_secs_f32();
    }

    pub fn exit(&mut self) {
        self.last_duration = Instant::now().duration_since(self.start_time).as_secs_f32();
    }
}

impl Default for PerfData {
    fn default() -> Self {
        Self {
            last_rate_time: Instant::now(),
            start_time: Instant::now(),
            beginning: Instant::now(),
            time_tag: 0.0,
            last_rate: Default::default(),
            last_duration: Default::default(),
        }
    }
}

impl Display for PerfData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "rate: {}, duration: {}",
            self.last_rate, self.last_duration
        ))
    }
}
