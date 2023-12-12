use cfe::{
    self,
    msg::{EventSeverity, ExampleOut, SbEvent, SbMsgData},
    Cfe,
};

pub struct Example {
    pub cf: Cfe,
    pub out: ExampleOut,
}

impl cfe::SbApp for Example {
    fn init(&mut self) {
        self.cf.log(
            SbEvent::AppInit,
            EventSeverity::Info,
            &format!("App {:?} initialized", self.cf.app_name),
        );
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                match msg.data {
                    SbMsgData::Sch1Hz => {
                        self.cf.log(
                            SbEvent::ExampleRun,
                            EventSeverity::Info,
                            &format!("got msg {:?}", msg),
                        );
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
