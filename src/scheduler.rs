use std::collections::VecDeque;

use crate::{os::Os, proc::PId};

pub trait Scheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId);

    fn switch_process(&mut self, os: &mut Os);

    fn on_tick(&mut self, _os: &mut Os) {}
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

struct SJFScheduler {}
struct STCFScheduler {}
struct RoundRobinScheduler {}
struct MLFQScheduler {}
struct FairShareScheduler {}
