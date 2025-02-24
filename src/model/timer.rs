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

    pub fn remaining_time(&self) -> Duration {
        if self.is_completed() {
            Duration::from_secs(0)
        } else {
            self.duration - self.start_time.elapsed()
        }
    }
}
