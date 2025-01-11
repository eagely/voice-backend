use std::sync::Arc;

use crate::handlers::traits::{ProcessingHandler, RecordingHandler, TranscriptionHandler};

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
