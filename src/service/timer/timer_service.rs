use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait TimerService: Send + Sync {
    async fn set(&self, duration: Duration, description: String) -> Result<String>;
}
