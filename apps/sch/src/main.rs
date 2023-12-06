use std::{time::{Duration, Instant}, thread::park_timeout, net::UdpSocket};

use cfe::{self, SbApp, udp::SbUdp};
use log::*;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    simple_log::quick!("info");
    let mut sch = Sch{
        cf: cfe::Cfe::init_cfe(),
        out: cfe::msg::SchOut::default(),
    };
    let udp = UdpSocket::bind("0.0.0.0:5123")?;
    udp.set_nonblocking(true)?;
    sch.cf.add_connection(Box::new(SbUdp::new(udp, "127.0.0.1:3412")?));
    sch.cf.send_message(cfe::msg::SbMsg::SchOut(cfe::msg::SchOut::default()));
    sch.run();
    return Ok(());
}

struct Sch {
    cf: cfe::Cfe,
    out: cfe::msg::SchOut
}


impl cfe::SbApp for Sch {
    fn init(&mut self) {
        info!("starting SCH");
    }
    fn start(&mut self) {
        let interval = Duration::from_millis(10);
        let mut next_time = Instant::now() + interval;
    
        loop {
            park_timeout(next_time.duration_since(Instant::now()));
            self.out.perf.enter();
    
            self.cf.send_message(cfe::msg::SbMsg::Sch100Hz);
            if self.out.perf.counter % 2 == 0 {
                self.cf.send_message(cfe::msg::SbMsg::Sch50Hz);
                
            }
            if self.out.perf.counter % 4 == 0 {
                self.cf.send_message(cfe::msg::SbMsg::Sch25Hz);
                self.cf.recv_message(false);
            }
            if self.out.perf.counter % 10 == 0 {
                self.cf.send_message(cfe::msg::SbMsg::Sch10Hz);
            }
            if self.out.perf.counter % 20 == 0 {
                self.cf.send_message(cfe::msg::SbMsg::Sch5Hz);
            }
            if self.out.perf.counter % 100 == 0 {
                self.cf.send_message(cfe::msg::SbMsg::Sch1Hz);
                self.cf.send_message(cfe::msg::SbMsg::SchOut(self.out.clone()));
            }
    
            next_time += interval;
            self.out.perf.exit();
        }
    }
}