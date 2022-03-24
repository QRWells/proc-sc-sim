use std::sync::Arc;

pub type PId = usize;

#[derive(Debug)]
pub enum Task {
    IOBound(u64),
    CPUBound(u64),
}

#[derive(Debug)]
pub struct Job {
    pub statements: Vec<Task>,
    pub t_io: u64,
    pub t_cpu: u64,
    pub t_total: u64,
}

#[derive(Debug)]
pub struct Process {
    pub id: PId,
    pub job: Arc<Job>,
    t_arrive: u64,
    t_turnaround: u64,
    t_completion: u64,
    pub complete: bool,
}
