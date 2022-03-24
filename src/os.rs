use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    proc::{PId, Process},
    scheduler::Scheduler,
};

const MAX_PID: usize = 1 << 10;

pub struct Os {
    interval: u64,
    clock: u64,
    last_pid: PId,
    processes: HashMap<PId, Process>,
    running_process_pid: Option<PId>,
    scheduler: Arc<Mutex<Box<dyn Scheduler + Send>>>,
}

impl Os {
    pub fn new(interval: Option<u64>, scheduler: Arc<Mutex<Box<dyn Scheduler + Send>>>) -> Os {
        Os {
            interval: match interval {
                Some(x) => x,
                None => 1,
            },
            last_pid: 0,
            clock: 0,
            processes: HashMap::new(),
            running_process_pid: None,
            scheduler,
        }
    }

    pub fn add_proc(&mut self, process: Process) {
        self.processes.insert(self.last_pid, process);
        self.last_pid += 1;
        if self.last_pid >= MAX_PID {
            panic!("no more process could be added!")
        }
    }

    pub fn run(&mut self) {
        while !self.is_completed() {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        self.clock += self.interval;
        let scheduler = self.scheduler.clone();
        let mut scheduler = scheduler.lock().expect("lock failed");
        scheduler.on_tick(self);
    }

    pub fn is_completed(&self) -> bool {
        self.processes.iter().all(|(_, v)| v.complete)
    }
}
