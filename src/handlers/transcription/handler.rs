use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TranscriptionHandler: Send + Sync {
    async fn transcribe(&self, audio: Vec<u8>) -> Result<String>;
}
