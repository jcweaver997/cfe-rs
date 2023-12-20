use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use cfe::msg::{EventSeverity, SbEvent, SbMsgData};

pub struct Sch {
    pub cf: cfe::Cfe,
    pub out: cfe::msg::SchOut,
}

impl cfe::SbApp for Sch {
    fn init(&mut self) {
        self.cf.log(
            SbEvent::AppInit,
            EventSeverity::Info,
            &format!("App {:?} initialized", self.cf.app_name),
        );
    }
    fn start(&mut self) {
        let interval = Duration::from_millis(10);
        let mut next_time = Instant::now() + interval;

        loop {
            let now = Instant::now();
            if Instant::now() > next_time {
                let mut skipped = 0;
                while Instant::now() > next_time {
                    next_time += interval;
                    skipped += 1;
                }
                self.cf.log(
                    SbEvent::SchBroke(skipped),
                    EventSeverity::Warn,
                    &format!("sch behind, skipping {} cycles", skipped),
                );
            }
            sleep(next_time - now);

            self.out.perf.enter();
            self.cf.send_message(SbMsgData::Sch100Hz);
            if self.out.perf.counter % 2 == 0 {
                self.cf.send_message(SbMsgData::Sch50Hz);
            }
            if self.out.perf.counter % 4 == 0 {
                self.cf.send_message(SbMsgData::Sch25Hz);
                self.cf.recv_message(false);
            }
            if self.out.perf.counter % 10 == 0 {
                self.cf.send_message(SbMsgData::Sch10Hz);
                self.cf.send_message(SbMsgData::SchOut(self.out.clone()));
            }
            if self.out.perf.counter % 20 == 0 {
                self.cf.send_message(SbMsgData::Sch5Hz);
            }
            if self.out.perf.counter % 100 == 0 {
                self.cf.send_message(SbMsgData::Sch1Hz);
            }

            next_time += interval;
            self.out.perf.exit();
        }
    }
}
