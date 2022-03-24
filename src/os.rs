use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    hash_wheel::HashWheel,
    proc::{PId, Process},
    scheduler::Scheduler,
};

pub struct Os {
    clock: u64,
    processes: HashMap<PId, Process>,
    waiting: HashWheel<PId>,
    running_process_pid: Option<PId>,
    scheduler: Arc<Mutex<Box<dyn Scheduler + Send>>>,
}

impl Os {
    pub fn run(&mut self) {
        while !self.is_completed() {
            self.tick();
        }
    }
    pub fn tick(&mut self) {}

    pub fn is_completed(&self) -> bool {
        true
    }
}
