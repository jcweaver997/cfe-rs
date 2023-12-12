use std::{
    sync::{Arc, Condvar, Mutex},
    thread::spawn,
};

use queues::{Buffer, IsQueue};

use crate::{msg::SbMsg, TCfeConnection};

pub struct SbMem {
    send: Arc<(Mutex<Buffer<SbMsg>>, Condvar)>,
    recv: Arc<(Mutex<Buffer<SbMsg>>, Condvar)>,
}

impl SbMem {
    pub fn create_pair(capacity: usize) -> (Self, Self) {
        let a = Arc::new((Mutex::new(Buffer::new(capacity)), Condvar::new()));
        let b = Arc::new((Mutex::new(Buffer::new(capacity)), Condvar::new()));
        return (
            Self {
                send: a.clone(),
                recv: b.clone(),
            },
            Self { send: b, recv: a },
        );
    }

    fn recv_thread(
        wait_read: Arc<(Mutex<Buffer<SbMsg>>, Condvar)>,
        msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>,
        token: usize,
    ) {
        loop {
            let recv = wait_read.0.lock().expect("failed to acquire mutex");
            let mut recv = wait_read.1.wait(recv).expect("failed to wait for read");
            let msg = if let Some(msg) = recv.remove().ok() {
                msg
            } else {
                continue;
            };
            msg_queue
                .0
                .lock()
                .expect("failed to acquire mutex")
                .add((msg, token));
            msg_queue.1.notify_one();
        }
    }
}

impl TCfeConnection for SbMem {
    fn send_message(&mut self, msg: &crate::msg::SbMsg) {
        self.send
            .0
            .lock()
            .expect("failed to acquire mutex")
            .add(msg.clone());
        self.send.1.notify_one();
    }

    fn start_recv(
        &mut self,
        msg_queue: Arc<(Mutex<Buffer<(SbMsg, usize)>>, Condvar)>,
        token: usize,
    ) {
        let wr = self.recv.clone();
        spawn(move || {
            Self::recv_thread(wr, msg_queue, token);
        });
    }
}
