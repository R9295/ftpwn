
use std::{
    sync::{Arc, Condvar, Mutex},
};


#[derive(Clone)]
pub struct HandleThreads {
    pair: Arc<(Mutex<bool>,Condvar)>,
}

pub trait ManageThreads {
    fn new() -> Self;
    fn finish_work(&self) -> bool;
    fn start_work(&self) -> bool;
}


impl ManageThreads for HandleThreads {
    fn new() -> Self {
        HandleThreads { pair:  Arc::new((Mutex::new(false), Condvar::new())) }
    }

    fn start_work(&self) -> bool {
        let &(ref lock, ref cvar) = &*self.pair;
        let mut done = lock.lock().unwrap();
        while !*done {
            done = cvar.wait(done).unwrap();
        }
        return true
    }

    fn finish_work(&self) -> bool {
        let &(ref lock, ref cvar) = &*self.pair.clone();
        let mut done = lock.lock().unwrap();
        *done = true;
        cvar.notify_one();
        return true
    }
}