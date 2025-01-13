use tokio::time::sleep;

use crate::error::Result;
use crate::handlers::{ProcessingHandler, RecordingHandler, TranscriptionHandler};
use std::{sync::Arc, time::Duration};

pub struct TcpServer {
    port: u16,
    recorder: Arc<dyn RecordingHandler>,
    transcriber: Arc<dyn TranscriptionHandler>,
    processor: Arc<dyn ProcessingHandler>,
}

impl TcpServer {
    pub fn new(
        port: u16,
        recorder: Arc<dyn RecordingHandler>,
        transcriber: Arc<dyn TranscriptionHandler>,
        processor: Arc<dyn ProcessingHandler>,
    ) -> Self {
        Self {
            port,
            recorder,
            transcriber,
            processor,
        }
    }

    pub async fn run(&self) -> Result<String> {
        self.recorder.start()?;
        sleep(Duration::from_secs(5)).await;
        let audio = self.recorder.stop()?;
        self.transcriber.transcribe(&audio).await
    }
}
