use std::{
    collections::HashMap,
    default,
    rc::{self, Rc},
    sync::{Arc, Mutex},
};

use crate::{
    proc::{PId, Process, ProcessState},
    scheduler::Scheduler,
    timer::hashed_wheel::HashedWheel,
};

use itertools::Itertools;

const MAX_PID: usize = 1 << 10;

pub struct Os {
    interval: u64,
    clock: u64,
    waiting_list: HashedWheel<PId>,
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
            clock: 0,
            waiting_list: HashedWheel::new(),
            processes: HashMap::new(),
            running_process_pid: None,
            scheduler,
        }
    }

    pub fn add_proc(&mut self, process: &mut Process) {
        let pid = self.generate_pid();
        process.set_pid(pid);
        self.processes.insert(pid, process.to_owned());
    }

    pub fn get_proc(&mut self, pid: &PId) -> &Process {
        &self.processes[pid]
    }

    pub fn current_proc(&self) -> Option<&Process> {
        self.running_process_pid
            .and_then(|pid| Some(&self.processes[&pid]))
    }

    pub fn run(&mut self) {
        while !self.is_completed() {
            self.tick();
        }
    }

    pub fn step(&mut self) {
        if !self.is_completed() {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        self.clock += self.interval;
        self.waiting_list.tick();

        let scheduler = self.scheduler.clone();
        let mut scheduler = scheduler.lock().expect("lock failed");
        scheduler.on_tick(self);

        self.processes.retain(|_, p| match p.state {
            ProcessState::Terminated => false,
            _ => true,
        });
    }

    pub fn is_completed(&self) -> bool {
        self.processes.is_empty() || self.processes.iter().all(|(_, v)| v.complete)
    }

    pub fn switch_proc(&mut self, pid: Option<PId>) {
        match pid {
            Some(pid) => {
                if let Some(cur_pid) = self.running_process_pid {
                    if cur_pid == pid {
                        self.processes.get_mut(&pid).unwrap().state = ProcessState::Waiting;
                    }
                    if self.processes.contains_key(&pid) {
                        self.running_process_pid = Some(pid);
                        self.processes.get_mut(&pid).unwrap().state = ProcessState::Running;
                    }
                }
            }
            None => self.running_process_pid = None,
        }
    }

    pub fn await_proc(&mut self, pid: PId, duration: u64) {
        self.waiting_list
            .add_timeout(pid, duration.try_into().unwrap());
    }

    pub fn expired_timeout(&mut self) -> Option<PId> {
        self.waiting_list.expire_timeout()
    }

    pub fn is_proc_running(&self, pid: PId) -> bool {
        match self.running_process_pid {
            Some(id) => id == pid,
            None => false,
        }
    }

    pub fn complete_proc(&mut self, pid: PId) {
        if self.processes.contains_key(&pid) {
            self.processes
                .get_mut(&pid)
                .unwrap()
                .set_complete(self.clock);
        }
    }

    fn generate_pid(&mut self) -> PId {
        let mut pid = 1;
        for i in self.processes.keys().sorted() {
            if pid == *i {
                pid += 1;
            }
        }
        if pid >= MAX_PID {
            panic!("no more process could be added!")
        }
        pid
    }
}
