use std::{time::{Duration, Instant}, thread::park_timeout, net::UdpSocket};

use cfe::{self, SbApp, udp::SbUdp, msg::SchOut};
use log::*;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    simple_log::quick!("info");
    let mut example = Example{
        cf: cfe::Cfe::init_cfe(),
        out: cfe::msg::ExampleOut::default()
    };
    let udp = UdpSocket::bind("0.0.0.0:3412")?;
    udp.set_nonblocking(true)?;
    example.cf.add_connection(Box::new(SbUdp::new(udp, "127.0.0.1:5123")?));
    example.run();
    return Ok(());
}

struct Example {
    cf: cfe::Cfe,
    out: cfe::msg::ExampleOut
}


impl cfe::SbApp for Example {
    fn init(&mut self) {
        info!("starting EXAMPLE");
        self.cf.subscribe(cfe::msg::SbMsg::Sch1Hz);
        self.cf.subscribe(cfe::msg::SbMsg::SchOut(SchOut::default()));
        
    }
    fn start(&mut self) {
    
        loop {
            let msg = self.cf.recv_message(true);
            info!("got msg {:?}", msg);
            self.out.perf.enter();
            self.out.perf.exit();
        }
    }
}