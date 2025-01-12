use super::handler::TranscriptionHandler;
use crate::error::Result;
use async_trait::async_trait;

pub struct LocalWhisperTranscriber {
    pub model: String,
}

#[async_trait]
impl TranscriptionHandler for LocalWhisperTranscriber {
    async fn transcribe(&self, audio: Vec<u8>) -> Result<String> {
        todo!()
    }
}
