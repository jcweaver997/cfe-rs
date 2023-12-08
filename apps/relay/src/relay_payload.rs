use cfe::{
    msg::{AppName, Computer},
    sbn::{udp::SbUdp, unix::SbUnix},
    Cfe, CfeConnection, SbApp,
};

use crate::relay::Relay;

mod relay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_log::quick!("info");
    let mut relay = Relay {
        cf: Cfe::init_cfe(Computer::Payload, AppName::Relay),
    };
    relay.cf.relay = true;

    let mut sch = CfeConnection::new(Box::new(SbUnix::new(
        "/tmp/pl-relay-sch.sock",
        "/tmp/pl-sch.sock",
    )?));
    sch.subscribe(cfe::msg::SbMsgData::Sch1Hz, Computer::Payload);
    relay.cf.add_connection(sch);

    relay
        .cf
        .add_connection(CfeConnection::new(Box::new(SbUdp::new(
            "0.0.0.0:41001".parse()?,
            "127.0.0.1:40001".parse()?,
        )?)));

    relay
        .cf
        .add_connection(CfeConnection::new(Box::new(SbUnix::new(
            "/tmp/pl-relay-example.sock",
            "/tmp/pl-example.sock",
        )?)));

    relay.run();
    return Ok(());
}
