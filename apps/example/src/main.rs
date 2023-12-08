use cfe::{
    self,
    msg::{Computer, ExampleOut, SbMsgData, SchOut},
    sbn::unix::SbUnix,
    Cfe, CfeConnection, SbApp,
};
use log::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_log::quick!("info");
    let mut example = Example {
        cf: Cfe::init_cfe(Computer::Payload, cfe::msg::AppName::Example),
        out: ExampleOut::default(),
    };
    let relay = SbUnix::new("/tmp/pl-example.sock", "/tmp/pl-relay-example.sock")?;
    let mut relay_con = CfeConnection::new(Box::new(relay));
    relay_con.subscribe(SbMsgData::Sch1Hz, Computer::Payload);
    example.cf.add_connection(relay_con);
    example.run();
    return Ok(());
}

struct Example {
    cf: Cfe,
    out: ExampleOut,
}

impl cfe::SbApp for Example {
    fn init(&mut self) {
        info!("starting {:?}", self.cf.app_name);
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                match msg.data {
                    SbMsgData::Sch1Hz => {
                        info!("got msg {:?}", msg);
                        self.out.perf.enter();

                        self.cf
                            .send_message(SbMsgData::ExampleOut(self.out.clone()));
                        self.out.perf.exit();
                    }
                    _ => {}
                }
            }
        }
    }
}
