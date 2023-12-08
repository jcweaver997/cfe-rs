use cfe::{Cfe, msg::{SbMsgData, RelayOut, RelayOutConnectionStatus}};
use log::*;

pub struct Relay {
    pub cf: Cfe,
}

impl cfe::SbApp for Relay {
    fn init(&mut self) {
        info!("starting Relay");
    }
    fn start(&mut self) {
        loop {
            let msg = self.cf.recv_message(true);
            if let Some(msg) = msg {
                if matches!(msg.data, SbMsgData::Sch1Hz) && msg.computer == self.cf.computer {
                    
                    let out = RelayOut{
                        connection_status: self.cf.connections.iter().map(|c|RelayOutConnectionStatus{
                            computer: c.computer,
                            app_name: c.app_name,
                            heartbeating: c.connected
                        }).collect()
                    };
                    self.cf.send_message(SbMsgData::RelayOut(out));
                }
            }
        }
    }
}
