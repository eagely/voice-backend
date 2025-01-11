use crate::handlers::{ProcessingHandler, RecordingHandler, TranscriptionHandler};
use std::sync::Arc;

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
}
