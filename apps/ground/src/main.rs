use std::time::Duration;

use log::*;

use cfe::{
    self,
    msg::{Computer, ExampleOut, SbMsgData, SchOut, RelayOut, EventSeverity, SbEvent},
    sbn::udp::SbUdp,
    Cfe, CfeConnection, SbApp,
};
use serde::{Serialize, Deserialize};
use serde_json::Value;

fn sub_all(cfe_con: &mut CfeConnection, computer: Computer) {
    cfe_con.subscribe(SbMsgData::ErrorMsg(SbEvent::None), computer);
    cfe_con.subscribe(SbMsgData::WarnMsg(SbEvent::None), computer);
    cfe_con.subscribe(SbMsgData::InfoMsg(SbEvent::None), computer);
    cfe_con.subscribe(SbMsgData::ExampleOut(ExampleOut::default()), computer);
    cfe_con.subscribe(SbMsgData::SchOut(SchOut::default()), computer);
    cfe_con.subscribe(SbMsgData::RelayOut(RelayOut::default()), computer);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_log::quick!("info");
    let mut ground = Ground {
        cf: Cfe::init_cfe(Computer::Ground, cfe::msg::AppName::Ground, cfe::msg::EventSeverity::Info),
        client: reqwest::blocking::Client::new()
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
    client: reqwest::blocking::Client
}

#[derive(Serialize, Deserialize)]
struct GroundTelemetryPoint {
    #[serde(rename = "RemoteTime")]
    remote_time: String,
    #[serde(rename = "LocalTime")]
    local_time: String,
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "App")]
    app: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Sequence")]
    sequence: i32,
    #[serde(rename = "Type")]
    data_type: i32,
    #[serde(rename = "IntData")]
    int_data: i64,
    #[serde(rename = "FloatData")]
    float_data: f64,
    #[serde(rename = "StringData")]
    string_data: String,
    #[serde(rename = "Severity")]
    severity: String

}

impl cfe::SbApp for Ground {
    fn init(&mut self) {
        self.cf.log(SbEvent::AppInit, EventSeverity::Info, &format!("App {:?} initialized", self.cf.app_name));
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                // self.cf.log(SbEvent::None, EventSeverity::Info, &format!("got msg {:?}", msg));
                let Ok(json_msg) = serde_json::to_string(&msg) else {
                    continue;
                };
                let Ok(json_obj) = serde_json::from_str::<Value>(&json_msg) else {
                    continue;
                };
                let mut telems = Vec::new();
                let flattened = flatten(String::new(), json_obj);
                for v in flattened {
                    
                    if v.0.starts_with("data."){
                        let now = chrono::Local::now();
                        telems.push(GroundTelemetryPoint {
                            remote_time: now.format("%G-%m-%dT%H:%M:%S.%f").to_string(),
                            local_time: now.format("%G-%m-%dT%H:%M:%S.%f").to_string(),
                            source: "cfe-rs".to_string(),
                            app: format!("{:?}.{:?}", msg.computer, msg.app_name),
                            name: v.0.chars().skip(5).collect::<String>(),
                            sequence: msg.sequence as i32,
                            data_type: match &v.1 {
                                Value::Number(_) => 15,
                                Value::String(_) => 1,
                                _ => 6
                            },
                            int_data:  match &v.1 {
                                Value::Number(n) => n.as_f64().unwrap() as i64,
                                _ => 0
                            },
                            float_data: match &v.1 {
                                Value::Number(n) => n.as_f64().unwrap(),
                                _ => 0.0
                            },
                            string_data: v.1.to_string(),
                            severity: "INFO".to_string(),
                        });
                    }
                }

                self.client.post("http://10.0.1.20:5900/api/Telemetry").json(&telems).send().unwrap();
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
                    format!("{}.{}", p, obj.0)
                };
                r.extend(flatten(path, obj.1));
            }
            return r;
        },
        Value::Array(a) => {
            let mut r = Vec::new();
            for i in 0..a.len() {
                let path = format!("{}.{}", p, i);
                r.extend(flatten(path, a[i].clone()));
            }
            return r;
        },
        Value::String(s) => {
            return vec![(p, Value::String(s))];
        }
        _ => {}
    }
    return Vec::new();
}
