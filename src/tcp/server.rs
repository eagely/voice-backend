use tokio::time::sleep;

use crate::error::Result;
use crate::handlers::{ProcessingHandler, RecordingHandler, TranscriptionHandler};
use std::{sync::Arc, time::Duration};

pub struct TcpServer {
    port: u16,
    recorder: Arc<dyn RecordingHandler>,
    recognizer: Arc<dyn TranscriptionHandler>,
    processor: Arc<dyn ProcessingHandler>,
}

impl TcpServer {
    pub fn new(
        port: u16,
        recorder: Arc<dyn RecordingHandler>,
        recognizer: Arc<dyn TranscriptionHandler>,
        processor: Arc<dyn ProcessingHandler>,
    ) -> Self {
        Self {
            port,
            recorder,
            recognizer,
            processor,
        }
    }

    pub async fn run(&self) -> Result<Vec<u8>> {
        self.recorder.start()?;
        sleep(Duration::from_secs(5)).await;
        self.recorder.stop()
    }
}
