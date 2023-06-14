use std::thread;

use esp_idf_sys as _;
use log::*;

use apps::*;
mod wifi_driver;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    start();
}

fn start() {
    info!("Starting apps...");
    let handle = thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(sch::start)
        .unwrap();

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(example::start)
        .unwrap();

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(to::start)
        .unwrap();

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(|| {
            tcp::start(tcp::Init {
                bind_addr: "0.0.0.0:5011".to_string(),
                peer_addr: "192.168.4.66:5010".to_string(),
                default_enabled: false,
            })
        })
        .unwrap();
    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(wifi_driver::start)
        .unwrap();
    handle.join().ok();
}
