use std::collections::HashSet;

use crate::perf::PerfData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Computer{
    #[default]
    None,
    Flight,
    Payload,
    Ground
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppName{
    #[default]
    None,
    Relay,
    Sch,
    Example,
    Ground
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ErrorMsg{
    #[default]
    None
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SbMsgData {
    #[default]
    None, // Used as heartbeat
    SbSubReq(SbSubReq),
    SchOut(SchOut),
    Sch100Hz,
    Sch50Hz,
    Sch25Hz,
    Sch10Hz,
    Sch5Hz,
    Sch1Hz,
    ExampleOut(ExampleOut),
    RelayOut(RelayOut),
    ErrorMsg

}

impl SbMsgData {
    pub fn get_id(&self) -> u64{
        return unsafe { std::mem::transmute(std::mem::discriminant(self)) };
    }

}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SbMsg {
    pub data: SbMsgData,
    pub computer: Computer,
    pub app_name: AppName,
    pub sequence: u16
}

impl SbMsg {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>>{
        return bincode::serialize(self);
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        return bincode::deserialize(bytes)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SbSubReq {
    pub subs: HashSet<(u64, Computer)>,
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


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayOutConnectionStatus {
    pub computer: Computer,
    pub app_name: AppName,
    pub heartbeating: bool

}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayOut {
    pub connection_status: Vec<RelayOutConnectionStatus>
}