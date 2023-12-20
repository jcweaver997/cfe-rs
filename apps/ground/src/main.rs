use std::time::Duration;

use cfe::{
    self,
    msg::{Computer, ExampleOut, SbMsgData, SchOut, RelayOut, EventSeverity, SbEvent},
    sbn::udp::SbUdp,
    Cfe, CfeConnection, SbApp,
};
use serde_json::Value;

fn sub_all(cfe_con: &mut CfeConnection, computer: Computer) {
    cfe_con.subscribe(SbMsgData::ErrorMsg(SbEvent::None), computer);
    cfe_con.subscribe(SbMsgData::WarnMsg(SbEvent::None), computer);
    cfe_con.subscribe(SbMsgData::InfoMsg(SbEvent::None), computer);
    // cfe_con.subscribe(SbMsgData::ExampleOut(ExampleOut::default()), computer);
    cfe_con.subscribe(SbMsgData::SchOut(SchOut::default()), computer);
    // cfe_con.subscribe(SbMsgData::RelayOut(RelayOut::default()), computer);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ground = Ground {
        cf: Cfe::init_cfe(Computer::Ground, cfe::msg::AppName::Ground, cfe::msg::EventSeverity::Info),
    };

    let mut udp_con = CfeConnection::new(Box::new(SbUdp::new(
        "0.0.0.0:51001".parse()?,
        "127.0.0.1:50001".parse()?,
    )?));

    sub_all(&mut udp_con, Computer::Flight);
    sub_all(&mut udp_con, Computer::Payload);
    ground.cf.add_connection(udp_con);
    ground.run();
    return Ok(());
}

struct Ground {
    cf: Cfe,
}

impl cfe::SbApp for Ground {
    fn init(&mut self) {
        self.cf.log(SbEvent::AppInit, EventSeverity::Info, &format!("App {:?} initialized", self.cf.app_name));
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                self.cf.log(SbEvent::None, EventSeverity::Info, &format!("got msg {:?}", msg));
                let Ok(json_msg) = serde_json::to_string(&msg) else {
                    continue;
                };
                let Ok(json_obj) = serde_json::from_str::<Value>(&json_msg) else {
                    continue;
                };
                let flattened = flatten(String::new(), json_obj);
            }
        }
    }
}

fn flatten(p: String, v: Value) -> Vec<(String, Value)> {
    match v {
        Value::Bool(x) => return vec![(p, Value::Bool(x))],
        Value::Number(x) => return vec![(p, Value::Number(x))],
        Value::Object(o) => {
            let mut r = Vec::new();
            for obj in o {
                let path = if p.len() == 0 {
                    obj.0
                } else {
                    format!("{}/{}", p, obj.0)
                };
                r.extend(flatten(path, obj.1));
            }
            return r;
        },
        Value::Array(a) => {
            let mut r = Vec::new();
            for i in 0..a.len() {
                let path = format!("{}/{}", p, i);
                r.extend(flatten(path, a[i].clone()));
            }
            return r;
        }
        _ => {}
    }
    return Vec::new();
}
