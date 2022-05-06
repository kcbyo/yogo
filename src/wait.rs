use std::time::{Duration, Instant};

const DEFAULT_WAIT_TIME: u64 = 500;

#[derive(Clone, Debug)]
pub struct Waiter {
    duration: Duration,
    epoch: Instant,
    should_wait: bool,
}

impl Waiter {
    pub fn new() -> Self {
        Waiter {
            duration: Duration::from_millis(DEFAULT_WAIT_TIME),
            epoch: Instant::now(),
            should_wait: false,
        }
    }

    pub fn wait(&mut self) {
        if self.should_wait {
            let now = Instant::now();
            let diff = now - self.epoch;
            if diff < self.duration {
                std::thread::sleep(self.duration - diff);
            }
        } else {
            self.should_wait = true;
        }
        self.epoch = Instant::now();
    }
}

impl Default for Waiter {
    fn default() -> Self {
        Waiter::new()
    }
}
