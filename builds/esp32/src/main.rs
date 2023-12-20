use std::thread::{self, sleep};
use std::time::Duration;

use cfe::sbn::udp_server::SbUdpServer;
use cfe::{
    msg::{AppName, Computer, ExampleOut, SchOut},
    sbn::mem::SbMem,
    Cfe, CfeConnection, SbApp,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use example::Example;
use relay::Relay;
use sch::Sch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: std::env!("WIFI_SSID").into(),
            password: std::env!("WIFI_PASSWORD").into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::new(1, 0));
    }
    println!("Should be connected now");
    loop {
        println!(
            "IP info: {:?}",
            wifi_driver.sta_netif().get_ip_info().unwrap()
        );
        sleep(Duration::new(1, 0));
        if !wifi_driver
            .sta_netif()
            .get_ip_info()
            .unwrap()
            .ip
            .is_unspecified()
        {
            break;
        }
    }

    let (pair1, pair2) = SbMem::create_pair(64);
    let (pair3, pair4) = SbMem::create_pair(64);

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            let mut relay = Relay {
                cf: Cfe::init_cfe(
                    Computer::Payload,
                    AppName::Relay,
                    cfe::msg::EventSeverity::Info,
                ),
            };
            relay.cf.relay = true;
            let mut sch = CfeConnection::new(Box::new(pair1));
            let mut example = CfeConnection::new(Box::new(pair3));
            sch.subscribe(cfe::msg::SbMsgData::Sch1Hz, Computer::Payload);
            example.subscribe(
                cfe::msg::SbMsgData::ExampleOut(ExampleOut::default()),
                Computer::Payload,
            );
            relay.cf.add_connection(sch);
            relay.cf.add_connection(example);
            relay.cf.add_connection(CfeConnection::new(Box::new(
                SbUdpServer::new("0.0.0.0:50001".parse().unwrap()).unwrap(),
            )));
            relay.run();
        })
        .unwrap();

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            let mut example: Example = Example {
                cf: Cfe::init_cfe(
                    Computer::Payload,
                    AppName::Example,
                    cfe::msg::EventSeverity::Info,
                ),
                out: ExampleOut::default(),
            };
            let mut relay = CfeConnection::new(Box::new(pair4));
            relay.subscribe(cfe::msg::SbMsgData::Sch1Hz, Computer::Payload);
            example.cf.add_connection(relay);
            example.run();
        })
        .unwrap();

    thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            let mut sch: Sch = Sch {
                cf: Cfe::init_cfe(
                    Computer::Payload,
                    AppName::Sch,
                    cfe::msg::EventSeverity::Info,
                ),
                out: SchOut::default(),
            };
            let schc = CfeConnection::new(Box::new(pair2));
            sch.cf.add_connection(schc);
            sch.run();
        })
        .unwrap();


    return Ok(());
}
