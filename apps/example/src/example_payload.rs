use cfe::{Cfe, msg::{Computer, ExampleOut, SbMsgData}, sbn::unix::SbUnix, CfeConnection, SbApp};
use example::Example;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut example = Example {
        cf: Cfe::init_cfe(Computer::Payload, cfe::msg::AppName::Example, cfe::msg::EventSeverity::Info),
        out: ExampleOut::default(),
    };
    let relay = SbUnix::new("/tmp/pl-example.sock", "/tmp/pl-relay-example.sock")?;
    let mut relay_con = CfeConnection::new(Box::new(relay));
    relay_con.subscribe(SbMsgData::Sch1Hz, Computer::Payload);
    example.cf.add_connection(relay_con);
    example.run();
    return Ok(());
}