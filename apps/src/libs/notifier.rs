use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

pub struct Notifier(Mutex<Vec<Notified>>);

impl Notifier {
    pub fn new() -> Self {
        Notifier(Mutex::new(Vec::new()))
    }
    pub fn subscribe(&self) -> Notified {
        let notified = Notified(Arc::new((Condvar::new(), Mutex::new(()))));
        self.0.lock().push(notified.clone());
        return notified;
    }

    pub fn notify(&self) {
        for n in self.0.lock().iter() {
            n.0 .0.notify_one();
        }
    }
}

#[derive(Clone)]
pub struct Notified(Arc<(Condvar, Mutex<()>)>);

impl Notified {
    pub fn wait(&self) {
        self.0 .0.wait(&mut self.0 .1.lock());
    }
}
