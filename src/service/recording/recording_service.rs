use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait RecordingService {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<Bytes>;
}
