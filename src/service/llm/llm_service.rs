use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn request(&self, input: &str) -> Result<String>;
}
