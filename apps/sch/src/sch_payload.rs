use cfe::{
    self,
    msg::{self, SbMsgData},
    sbn::unix::SbUnix,
    CfeConnection, SbApp,
};
mod sch;
use crate::sch::Sch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_log::quick!("info");
    let mut sch = Sch {
        cf: cfe::Cfe::init_cfe(msg::Computer::Payload, msg::AppName::Sch),
        out: cfe::msg::SchOut::default(),
    };
    let relay = SbUnix::new("/tmp/pl-sch.sock", "/tmp/pl-relay-sch.sock")?;
    let relay_con = CfeConnection::new(Box::new(relay));
    sch.cf.add_connection(relay_con);

    sch.cf
        .send_message(SbMsgData::SchOut(cfe::msg::SchOut::default()));
    sch.run();
    return Ok(());
}
