use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait TtsService: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Bytes>;
}
