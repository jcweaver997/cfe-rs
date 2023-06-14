pub mod ci;
pub mod example;
pub mod libs;
pub mod perf;
pub mod sch;
pub mod tcp;
pub mod to;
pub mod wifi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SbMsg {
    #[default]
    None,
    ToOut(to::Out),
    ToCmd(to::Command),
    WifiOut(wifi::Out),
    TcpOut(tcp::Out),
    ExampleOut(example::Out),
}
