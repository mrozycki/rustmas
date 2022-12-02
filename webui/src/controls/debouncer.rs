use instant::{Duration, Instant};

pub struct Debouncer {
    delay: Duration,
    last_event: Instant,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            last_event: Instant::now() - delay,
        }
    }

    pub fn poll(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_event) < self.delay {
            false
        } else {
            self.last_event = now;
            true
        }
    }
}
