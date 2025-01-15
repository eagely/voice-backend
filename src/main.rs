mod error;
mod handlers;
mod tcp;

use crate::error::Result;
use handlers::{
    processing::LocalProcessor, recording::LocalRecorder, transcription::LocalWhisperTranscriber,
};
use std::sync::Arc;
use tcp::server::TcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let recorder = Box::new(LocalRecorder::new("pipewire")?);

    let recognizer = Arc::new(LocalWhisperTranscriber::new("base.bin")?);

    let processor = Arc::new(LocalProcessor {
        model: "gpt2".to_string(),
    });

    let server = TcpServer::new("127.0.0.1:8080", recorder, recognizer, processor)?;
    loop {
        server.listen().await?;
    }
}
