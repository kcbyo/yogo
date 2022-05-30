use std::{
    thread,
    time::{Duration, Instant},
};

const DEFAULT_WAIT_TIME: u64 = 500;

#[derive(Clone, Debug)]
pub struct Waiter {
    duration: Duration,
    epoch: Option<Instant>,
}

impl Waiter {
    pub fn new() -> Self {
        Self::with_wait(DEFAULT_WAIT_TIME)
    }

    pub fn with_wait(time_in_milliseconds: u64) -> Self {
        Self {
            duration: Duration::from_millis(time_in_milliseconds),
            epoch: None,
        }
    }

    pub fn wait(&mut self) {
        if let Some(epoch) = self.epoch.take() {
            let now = Instant::now();
            let diff = now - epoch;
            if diff < self.duration {
                thread::sleep(self.duration - diff);
            }
        }

        self.epoch = Some(Instant::now());
    }
}

impl Default for Waiter {
    fn default() -> Self {
        Waiter::new()
    }
}
