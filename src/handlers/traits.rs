use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RecordingHandler: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<Vec<u8>>;
}

#[async_trait]
pub trait TranscriptionHandler: Send + Sync {
    async fn transcribe(&self, audio: Vec<u8>) -> Result<String>;
}

#[async_trait]
pub trait ProcessingHandler: Send + Sync {
    async fn respond(&self, input: &str) -> Result<String>;
}
