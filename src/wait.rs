use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct Waiter {
    duration: Duration,
    should_wait: bool,
}

impl Waiter {
    pub fn new() -> Self {
        Waiter {
            duration: Duration::from_millis(250),
            should_wait: false,
        }
    }

    pub fn wait(&mut self) {
        if self.should_wait {
            std::thread::sleep(self.duration);
        } else {
            self.should_wait = true;
        }
    }
}

impl Default for Waiter {
    fn default() -> Self {
        Waiter::new()
    }
}
