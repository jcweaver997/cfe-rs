use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::libs::msg_queue::MsgOut;

pub const SSID: &str = "JCAL";
pub const PASSWORD: &str = "F!v3five";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Out {
    pub has_ip: bool,
}

lazy_static! {
    pub static ref OUT_DATA: MsgOut<Out, 1> = MsgOut::new();
}
