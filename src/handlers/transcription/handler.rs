use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait TranscriptionHandler: Send + Sync {
    async fn transcribe(&self, audio: &Bytes) -> Result<String>;
}
