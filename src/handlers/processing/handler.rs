use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ProcessingHandler: Send + Sync {
    async fn process(&self, input: &str) -> Result<String>;
}
