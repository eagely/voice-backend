use crate::error::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn request(&self, input: &str) -> Result<BoxStream<'static, Result<String>>>;
}
