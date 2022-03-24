use crate::{os::Os, proc::PId};

pub trait Scheduler {
    fn on_process_ready(&mut self, os: &mut Os, pid: PId);

    fn switch_process(&mut self, os: &mut Os);

    fn on_tick(&mut self, _os: &mut Os) {}
}
