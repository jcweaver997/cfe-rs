use cfe::{
    self,
    msg::{Computer, ExampleOut, SbMsgData, EventSeverity, SbEvent},
    sbn::unix::SbUnix,
    Cfe, CfeConnection, SbApp,
};

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

struct Example {
    cf: Cfe,
    out: ExampleOut,
}

impl cfe::SbApp for Example {
    fn init(&mut self) {
        self.cf.log(SbEvent::AppInit, EventSeverity::Info, &format!("App {:?} initialized", self.cf.app_name));
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                match msg.data {
                    SbMsgData::Sch1Hz => {
                        self.cf.log(SbEvent::ExampleRun, EventSeverity::Info, &format!("got msg {:?}", msg));
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
