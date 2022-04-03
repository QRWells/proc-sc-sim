use core::time;
use std::collections::{linked_list, LinkedList};

#[derive(Debug, Clone)]
pub struct HashedWheelTimeout<T>
where
    T: Clone,
{
    item: T,
    round: isize,
}

impl<T> HashedWheelTimeout<T>
where
    T: Clone,
{
    fn new(item: T, round: isize) -> Self {
        Self { item, round }
    }

    fn tick_a_round(&mut self) {
        self.round -= 1;
    }
}
#[derive(Debug, Clone)]
pub struct HashedWheelBucket<T>
where
    T: Clone,
{
    timeouts: LinkedList<HashedWheelTimeout<T>>,
}

impl<T> HashedWheelBucket<T>
where
    T: Clone,
{
    fn new() -> Self {
        Self {
            timeouts: LinkedList::new(),
        }
    }

    fn tick(&mut self) {
        self.timeouts.iter_mut().for_each(|t| t.tick_a_round());
    }

    fn empty(&self) -> bool {
        self.timeouts.is_empty()
    }

    fn add_timeout(&mut self, timeout: HashedWheelTimeout<T>) {
        self.timeouts.push_back(timeout);
    }

    fn expired_timeout(&mut self) -> Option<T> {
        if let Some(expire) = self.timeouts.front().and_then(|t| Some(t.round <= 0)) {
            self.timeouts.pop_front().and_then(|t| Some(t.item))
        } else {
            None
        }
    }
}
#[derive(Debug)]
pub struct HashedWheel<T>
where
    T: Clone,
{
    buckets: Vec<HashedWheelBucket<T>>,
    pub current_tick: isize,
    pub wheel_size: usize,
    pub resolution: usize,
}

impl<T> HashedWheel<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self::with_size_and_resolution(8, 1)
    }

    pub fn with_size_and_resolution(wheel_size: usize, resolution: usize) -> Self {
        let buckets = vec![HashedWheelBucket::<T>::new(); wheel_size];
        Self {
            buckets,
            current_tick: -1,
            wheel_size,
            resolution,
        }
    }

    pub fn empty(&self) -> bool {
        self.buckets.iter().all(|bucket| bucket.empty())
    }

    pub fn tick(&mut self) {
        self.current_tick += self.resolution as isize;
        self.buckets.iter_mut().for_each(|bucket| bucket.tick());
    }

    pub fn add_timeout(&mut self, value: T, deadline: isize) {
        let round = deadline / self.wheel_size as isize + 1;
        let timeout = HashedWheelTimeout::<T> { item: value, round };
        self.buckets[deadline as usize % self.wheel_size].add_timeout(timeout);
    }

    pub fn expire_timeout(&mut self) -> Option<T> {
        self.buckets[self.current_tick as usize % self.wheel_size].expired_timeout()
    }
}
