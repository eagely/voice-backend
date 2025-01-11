use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RecordingHandler: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<Vec<u8>>;
}
