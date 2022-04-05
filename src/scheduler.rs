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
impl Scheduler for RoundRobinScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        self.ready_queue.push_back(pid);
    }

    fn switch_process(&mut self, os: &mut Os) {
        os.switch_proc(self.ready_queue.pop_front());
    }

    fn on_process_burst(&mut self, os: &mut Os, pid: PId) {
        let used_time_slice = self.used_time_slice_map.get(&pid).unwrap_or(&0).clone();
        if used_time_slice >= self.time_slice && os.is_proc_running(pid) {
            self.ready_queue.push_back(pid);
            self.used_time_slice_map.insert(pid, 0);
            self.switch_process(os);
        } else {
            self.used_time_slice_map
                .insert(pid, used_time_slice + os.interval);
        }
    }
}

struct MLFQScheduler {
    ready_queues: [IndexSet<PId>; 3],
    used_time_slice_map: HashMap<PId, u64>,
    running_process: Option<(PId, usize)>,
    time_slices: [u64; 2],
}

impl MLFQScheduler {
    fn get_priority(&self, pid: PId) -> usize {
        self.running_process
            .and_then(|(p, priority)| (pid == p).then(|| priority))
            .unwrap_or_else(|| {
                self.ready_queues
                    .iter()
                    .enumerate()
                    .find_map(|(pr, q)| q.get(&pid).and(Some(pr)))
                    .unwrap_or(0)
            })
    }

    fn level_down(&mut self, pid: PId) {
        let pr = self.get_priority(pid);
        if pr >= self.ready_queues.len() - 1 {
            return;
        }
        self.ready_queues[pr].remove(&pid);
        self.ready_queues[pr + 1].insert(pid);
    }

    fn is_proc_running(&self, pid: PId) -> bool {
        self.running_process.map_or(false, |(id, _)| id == pid)
    }
}

impl Scheduler for MLFQScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        self.ready_queues[0].insert(pid);
    }

    fn switch_process(&mut self, os: &mut Os) {
        if let Some((pid, pr)) = self
            .ready_queues
            .iter_mut()
            .enumerate()
            .find_map(|(pri, q)| q.pop().map(|pid| (pid, pri)))
        {
            self.running_process = Some((pid, pr));
            os.switch_proc(Some(pid));
        } else {
            self.running_process = None;
            os.switch_proc(None);
        }
    }

    fn on_process_burst(&mut self, os: &mut Os, pid: PId) {
        let priority = self.get_priority(pid);
        let last_priority = self.ready_queues.len() - 1;
        if priority >= last_priority {
            if self.ready_queues[0..last_priority]
                .iter()
                .any(|q| !q.is_empty())
            {
                self.ready_queues[last_priority].insert(pid);
                self.switch_process(os);
            }
        } else {
            let used_time_slice = self.used_time_slice_map.get(&pid).copied().unwrap_or(0);
            if used_time_slice >= self.time_slices[priority] && self.is_proc_running(pid) {
                self.level_down(pid);
                self.used_time_slice_map.insert(pid, 0);
                self.switch_process(os);
            } else {
                self.used_time_slice_map
                    .insert(pid, used_time_slice + os.interval);
            }
        }
    }
}

struct FairShareScheduler {
    total_ticket: usize,
    next_pid: Option<PId>,
    process_ticket: HashMap<PId, usize>,
}

impl Scheduler for FairShareScheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId) {
        let ticket = os
            .get_proc(&pid)
            .and_then(|p| Some(p.priority))
            .unwrap_or(0) as usize
            * 100;
        self.total_ticket += ticket;
        self.process_ticket.insert(pid, ticket);
    }

    fn switch_process(&mut self, os: &mut Os) {
        os.switch_proc(self.next_pid);
    }

    fn on_process_burst(&mut self, os: &mut Os, pid: PId) {
        self.process_ticket.retain(|p, _| {
            os.get_proc(p)
                .and_then(|proc| Some(!proc.is_complete()))
                .unwrap_or(false)
        });
        self.total_ticket = self.process_ticket.values().sum();
        if self.process_ticket.len() == 0 {
            self.next_pid = None;
            self.switch_process(os);
            return;
        }

        let mut winner = rand::random::<usize>() % self.total_ticket + 1;
        for (p, t) in self.process_ticket.iter() {
            winner -= t;
            if winner > 0 {
                continue;
            }
            self.next_pid = Some(*p);
            self.switch_process(os);
            return;
        }
    }
}
