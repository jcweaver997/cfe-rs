use apps::libs::msg_queue::MsgQueue;
use apps::*;
use lazy_static::lazy_static;
use log::*;
use rerun::components::Scalar;
use rerun::MsgSender;
use ringbuf::Rb;
use serde_json::Value;
use std::thread::spawn;

lazy_static! {
    pub static ref MSGS_IN: MsgQueue<SbMsg, 10> = MsgQueue::new();
}

fn main() {
    simple_log::quick!("info");
    start();
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
        }
        _ => {}
    }
    return Vec::new();
}

fn start() {
    info!("Starting apps...");
    spawn(sch::start);
    spawn(ci::start);
    spawn(|| {
        tcp::start(tcp::Init {
            bind_addr: "0.0.0.0:5010".to_string(),
            peer_addr: "127.0.0.1:5011".to_string(),
            default_enabled: true,
        })
    });

    let recording = rerun::RecordingStreamBuilder::new("MMouse")
        .connect(rerun::default_server_addr())
        .unwrap();

    let n = sch::SCH50HZ.subscribe();

    ci::MSGS.subscribe(&MSGS_IN);

    loop {
        n.wait();
        for msg in MSGS_IN.lock().pop_iter() {
            let Ok(json_msg) = serde_json::to_string(&msg) else {
                continue;
            };
            let Ok(json_obj) = serde_json::from_str::<Value>(&json_msg) else {
                continue;
            };

            let flattened = flatten(String::new(), json_obj);
            for p in flattened {
                match p.1 {
                    Value::Bool(b) => {
                        MsgSender::new(p.0)
                            .with_splat(Scalar(if b { 1.0 } else { 0.0 }))
                            .unwrap()
                            .send(&recording)
                            .unwrap();
                    }
                    Value::Number(n) => {
                        MsgSender::new(p.0)
                            .with_splat(Scalar(n.as_f64().unwrap()))
                            .unwrap()
                            .send(&recording)
                            .unwrap();
                    }
                    _ => {}
                }
            }

            // match msg {
            //     SbMsg::ToOut(o) => {
            // MsgSender::new("to/perf/last_rate")
            //     .with_splat(Scalar(o.perf.last_rate as f64))
            //     .unwrap()
            //     .send(&recording)
            //     .unwrap();
            //         MsgSender::new("to/perf/last_duration")
            //             .with_splat(Scalar(o.perf.last_duration as f64))
            //             .unwrap()
            //             .send(&recording)
            //             .unwrap();
            //     }
            //     _ => {}
            // }
        }
    }
}
