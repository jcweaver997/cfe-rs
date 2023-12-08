use cfe::{
    msg::{AppName, Computer},
    sbn::{udp::SbUdp, udp_server::SbUdpServer, unix::SbUnix},
    Cfe, CfeConnection, SbApp,
};

use crate::relay::Relay;

mod relay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut relay = Relay {
        cf: Cfe::init_cfe(Computer::Flight, AppName::Relay, cfe::msg::EventSeverity::Info),
    };
    relay.cf.relay = true;

    let mut sch = CfeConnection::new(Box::new(SbUnix::new(
        "/tmp/fl-relay-sch.sock",
        "/tmp/fl-sch.sock",
    )?));
    sch.subscribe(cfe::msg::SbMsgData::Sch1Hz, Computer::Flight);
    relay
        .cf
        .add_connection(sch);


    relay
        .cf
        .add_connection(CfeConnection::new(Box::new(SbUdp::new(
            "0.0.0.0:40001".parse()?,
            "127.0.0.1:41001".parse()?,
        )?)));

    relay
        .cf
        .add_connection(CfeConnection::new(Box::new(SbUdpServer::new(
            "0.0.0.0:50001".parse()?,
        )?)));

    relay.run();

    return Ok(());
}
