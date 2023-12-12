use std::thread::{self, spawn};

use cfe::{
    msg::{AppName, Computer, SchOut, ExampleOut},
    sbn::mem::SbMem,
    Cfe, CfeConnection, SbApp,
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
            example.subscribe(cfe::msg::SbMsgData::ExampleOut(ExampleOut::default()), Computer::Payload);
            relay.cf.add_connection(sch);
            relay.cf.add_connection(example);
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
                out: ExampleOut::default()
            };
            let mut relay = CfeConnection::new(Box::new(pair4));
            relay.subscribe(cfe::msg::SbMsgData::Sch1Hz, Computer::Payload);
            example.cf.add_connection(relay);
            example.run();
        })
        .unwrap();


    
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

    return Ok(());
}
