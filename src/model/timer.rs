use std::time::{Duration, Instant};

pub struct Timer {
    pub duration: Duration,
    pub start_time: Instant,
    pub description: String,
}

impl Timer {
    pub fn new(duration: Duration, description: String) -> Self {
        Self {
            duration,
            start_time: Instant::now(),
            description,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}
