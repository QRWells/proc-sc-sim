use std::collections::VecDeque;

pub type PId = usize;

#[derive(Debug, Clone, Copy)]
pub enum Task {
    IOBound(u64),
    CPUBound(u64),
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    Runnable,
    Running,
    Waiting,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub id: PId,
    pub tasks: VecDeque<Task>,
    pub state: ProcessState,
    pub cpu: u32,

    pub name: Option<String>,
    pub arrive_time: u64,
    pub turnaround_time: Option<u64>,
    pub burst_time: u64,
    pub complete_time: Option<u64>,
    pub response_time: Option<u64>,
    pub remaining_time: u64,
    pub time_have_burst: u64,

    pub complete: bool,
}

impl Process {
    pub fn new(pid: PId, t_arrive: u64, burst_time: u64) -> Self {
        Self {
            name: None,
            id: pid,
            state: ProcessState::Runnable,
            cpu: 0,
            tasks: VecDeque::new(),
            arrive_time: t_arrive,
            turnaround_time: None,
            burst_time,
            complete_time: None,
            response_time: None,
            time_have_burst: 0,
            remaining_time: burst_time,
            complete: false,
        }
    }

    pub fn append_task(&mut self, task: Task) {
        self.tasks.push_back(task);
        match task {
            Task::IOBound(duration) | Task::CPUBound(duration) => self.burst_time += duration,
        }
    }

    pub(crate) fn set_pid(&mut self, pid: PId) {
        self.id = pid;
    }

    pub(crate) fn set_complete(&mut self, current_time: u64) {
        self.state = ProcessState::Terminated;
        self.complete_time = Some(current_time);
        self.turnaround_time = Some(current_time - self.arrive_time);
    }

    pub(crate) fn burst(&mut self, clock: u64) -> Option<Task> {
        if (self.time_have_burst == 0) {
            self.response_time = Some(clock - self.arrive_time - 1);
        }
        self.time_have_burst += 1;
        self.remaining_time -= 1;

        if self.time_have_burst >= self.burst_time {
            self.set_complete(clock);
            return None;
        }

        self.tasks.front_mut().and_then(|task| -> Option<Task> {
            match task {
                Task::IOBound(duration) => {
                    *duration -= 1;
                    Some(Task::IOBound(*duration))
                }
                Task::CPUBound(duration) => {
                    *duration -= 1;
                    Some(Task::CPUBound(*duration))
                }
            }
        })
    }

    pub(crate) fn bump_to_next(&mut self) -> Option<Task> {
        self.tasks.pop_front().and_then(|task| {
            match task {
                Task::IOBound(duration) | Task::CPUBound(duration) => {
                    self.time_have_burst += duration
                }
            }
            Some(task)
        })
    }

    pub(crate) fn is_complete(&self) -> bool {
        match self.state {
            ProcessState::Terminated => true,
            _ => false,
        }
    }
}
