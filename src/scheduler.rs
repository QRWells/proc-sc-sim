use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
};

use indexmap::IndexSet;
use priority_queue::PriorityQueue;

use crate::{
    os::Os,
    proc::{PId, Task},
};

pub trait Scheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId);

    fn switch_process(&mut self, os: &mut Os);

    fn on_tick(&mut self, os: &mut Os) {
        while let Some(p) = os.waiting_list.expire_timeout() {
            self.on_process_ready(os, p);
        }

        self.burst_proc(os);
    }

    fn burst_proc(&mut self, os: &mut Os) {
        let clock = os.clock;
        if let Some((new_statement, is_completed, pid)) = os
            .running_process()
            .map(|process| (process.burst(clock), process.is_complete(), process.id))
        {
            if let Some(new_statement) = new_statement {
                self.run_task(os, new_statement, pid);
            } else if is_completed {
                os.complete_proc(pid);
                if os.is_proc_running(pid) {
                    self.switch_process(os);
                }
            }
            self.on_process_burst(os, pid);
        } else {
            self.switch_process(os);
        }
    }

    fn run_task(&mut self, os: &mut Os, task: Task, pid: PId) {
        match task {
            Task::CPUBound(duration) => self.run_cpu_bound_task(os, duration, pid),
            Task::IOBound(duration) => self.run_io_bound_task(os, duration, pid),
        }
    }

    #[allow(unused)]
    fn run_cpu_bound_task(&mut self, os: &mut Os, duration: u64, pid: PId) {}
    fn run_io_bound_task(&mut self, os: &mut Os, duration: u64, pid: PId) {
        let clock = os.clock;
        let proc = os.get_proc(&pid);
        if let Some((pid, is_completed)) = proc.map(|process| {
            if let Some(next_statement) = process.bump_to_next() {
                (process.id, process.is_complete())
            } else {
                (0, false)
            }
        }) {
            if is_completed {
                os.complete_proc(pid);
            } else {
                os.await_proc(pid, duration);
            }
        }
        if os.is_proc_running(pid) {
            self.switch_process(os);
        }
    }

    // Used for preemptive
    #[allow(unused)]
    fn on_process_burst(&mut self, os: &mut Os, pid: PId) {}
}

struct FCFSScheduler {
    ready_queue: VecDeque<PId>,
}

impl Scheduler for FCFSScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        self.ready_queue.push_back(pid);
    }

    fn switch_process(&mut self, os: &mut Os) {
        os.switch_proc(self.ready_queue.pop_front());
    }
}

#[derive(Debug)]
struct SJFScheduler {
    ready_queue: PriorityQueue<PId, Reverse<u64>>,
}

impl Scheduler for SJFScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        if let Some(proc) = os.get_proc(&pid) {
            let p = proc.burst_time;
            self.ready_queue.push(pid, Reverse(p));
        }
    }

    fn switch_process(&mut self, os: &mut Os) {
        let pid = self.ready_queue.pop();
        os.switch_proc(pid.and_then(|p| Some(p.0)));
    }
}

struct STCFScheduler {
    ready_queue: PriorityQueue<PId, Reverse<u64>>,
}

impl Scheduler for STCFScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        if let Some(proc) = os.get_proc(&pid) {
            let p = proc.burst_time;
            self.ready_queue.push(pid, Reverse(p));
        }
    }

    fn switch_process(&mut self, os: &mut Os) {
        let pid = self.ready_queue.pop();
        os.switch_proc(pid.and_then(|p| Some(p.0)));
    }

    fn on_process_burst(&mut self, os: &mut Os, pid: PId) {
        let process_remaining_time = os.get_proc(&pid).map(|p| p.remaining_time).unwrap_or(0);
        if self
            .ready_queue
            .peek()
            .map_or(false, |(_, top_remaining_time)| {
                top_remaining_time.gt(&&Reverse(process_remaining_time))
            })
        {
            self.switch_process(os);
            self.ready_queue.push(pid, Reverse(process_remaining_time));
        }
    }
}
struct RoundRobinScheduler {
    ready_queue: VecDeque<PId>,
    used_time_slice_map: HashMap<PId, u64>,
    time_slice: u64,
}
struct MLFQScheduler {
    ready_queues: [IndexSet<PId>; 3],
    used_time_slice_map: HashMap<PId, u64>,
    running_process: Option<(PId, usize)>,
    time_slices: [u64; 2],
}
struct FairShareScheduler {
    total_ticket: usize,
    next_pid: PId,
    process_ticket: HashMap<PId, usize>,
}
