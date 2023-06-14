use std::thread;

use apps::*;
use log::*;

fn main() {
    simple_log::quick!("info");
    start();
}

fn start() {
    info!("Starting apps...");
    let handle = thread::Builder::new()
        .name("sch".into())
        .stack_size(16 * 1024)
        .spawn(sch::start)
        .unwrap();

    thread::Builder::new()
        .name("example".into())
        .stack_size(16 * 1024)
        .spawn(example::start)
        .unwrap();

    thread::Builder::new()
        .name("to".into())
        .stack_size(128 * 1024)
        .spawn(to::start)
        .unwrap();

    thread::Builder::new()
        .name("tcp".into())
        .stack_size(16 * 1024)
        .spawn(|| {
            tcp::start(tcp::Init {
                bind_addr: "0.0.0.0:5011".to_string(),
                peer_addr: "127.0.0.1:5010".to_string(),
                default_enabled: true,
            })
        })
        .unwrap();

    handle.join().ok();
}
