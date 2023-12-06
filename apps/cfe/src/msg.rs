use std::collections::HashSet;

use crate::perf::PerfData;
use serde::{Deserialize, Serialize};

pub enum Computer{
    Flight,
    Payload
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SbMsg {
    #[default]
    None, // Used as heartbeat
    SbSubReq(SbSubReq),
    SbSubRes(SbSubRes),
    SchOut(SchOut),
    Sch100Hz,
    Sch50Hz,
    Sch25Hz,
    Sch10Hz,
    Sch5Hz,
    Sch1Hz,
    ExampleOut(ExampleOut),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SbSubReq {
    pub subs: HashSet<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SbSubRes {
    pub subs: HashSet<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchOut {
    pub perf: PerfData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExampleOut {
    pub perf: PerfData,
}