use parking_lot::{Mutex, MutexGuard};
use ringbuf::{Rb, StaticRb};

pub struct MsgQueue<T, const N: usize>(Mutex<StaticRb<T, N>>);

impl<T, const N: usize> MsgQueue<T, N> {
    pub fn new() -> Self {
        MsgQueue(Mutex::new(StaticRb::<T, N>::default()))
    }

    pub fn lock(&self) -> MutexGuard<StaticRb<T, N>> {
        Mutex::lock(&self.0)
    }

    pub fn push(&self, msg: T) {
        self.lock().push_overwrite(msg);
    }
}

pub struct MsgOut<T: 'static, const N: usize>(Mutex<Vec<&'static MsgQueue<T, N>>>)
where
    T: Copy;
impl<T, const N: usize> MsgOut<T, N>
where
    T: Copy,
{
    pub fn new() -> Self {
        MsgOut(Mutex::new(Vec::new()))
    }

    pub fn subscribe(&self, subscriber: &'static MsgQueue<T, N>) {
        self.0.lock().push(subscriber);
    }

    pub fn send(&self, msg: T) {
        for sub in self.0.lock().iter() {
            let mut l = sub.lock();
            if l.free_len() == 0 {
                l.skip(1);
            }
            if l.push(msg).is_err() {
                log::error!("failed to push msg to queue");
            }
        }
    }

    pub fn send_slice(&self, msgs: &[T]) {
        for sub in self.0.lock().iter() {
            let mut l = sub.lock();
            let free_len = l.free_len();
            if free_len < msgs.len() {
                l.skip(msgs.len() - free_len);
            }
            l.push_slice(msgs);
        }
    }
}
