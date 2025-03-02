use super::timer_service::TimerService;
use crate::error::Result;
use crate::model::timer::Timer;
use async_trait::async_trait;
use notify_rust::Notification;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct MemoryTimer {
    timers: Arc<Mutex<Vec<Timer>>>,
}

impl MemoryTimer {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TimerService for MemoryTimer {
    async fn set(&self, duration: Duration, description: String) -> Result<String> {
        let timer = Timer::new(duration, description);
        let timers_clone = self.timers.clone();

        let (tx, mut rx) = channel(1);
        timers_clone.lock().await.push(timer);

        tokio::spawn(async move {
            sleep(duration).await;
            let mut timers = timers_clone.lock().await;
            if let Some(pos) = timers.iter().position(|t| t.is_completed()) {
                let result = Notification::new()
                    .summary("Timer completed")
                    .body(&timers[pos].description)
                    .show();

                timers.remove(pos);

                if let Err(e) = tx.send(result).await {
                    eprintln!("Failed to send result: {}", e);
                }
            }
        });

        if let Some(result) = rx.recv().await {
            result?;
        }
        Ok(format!("Timer set for {} seconds", duration.as_secs()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use std::time::Duration;

    #[tokio::test]
    async fn test_memory_timer() -> Result<()> {
        let timer_service = MemoryTimer::new();
        let description = "Test timer for 1 second".to_string();
        let result = timer_service.set(Duration::from_secs(1), description).await;
        assert!(result.is_ok());
        Ok(())
    }
}
