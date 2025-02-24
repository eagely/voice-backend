use super::timer_service::TimerService;
use crate::error::Result;
use crate::model::timer::Timer;
use async_trait::async_trait;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct MemoryTimer {
    timers: Arc<Mutex<Vec<Timer>>>,
}

impl MemoryTimer {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

#[async_trait]
impl TimerService for MemoryTimer {
    async fn set(&self, duration: Duration, description: String) -> Result<String> {
        let timer = Timer::new(duration, description);
        let timers_clone = self.timers.clone();

        timers_clone.lock().await.push(timer);

        tokio::spawn(async move {
            sleep(duration).await;
            let mut timers = timers_clone.lock().await;
            if let Some(pos) = timers.iter().position(|t| t.is_completed()) {
                timers.remove(pos);
            }
        });

        Ok(format!("Timer set for {} seconds", duration.as_secs()))
    }
}
