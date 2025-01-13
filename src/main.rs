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
    let recorder = Arc::new(LocalRecorder::new("pipewire")?);

    let recognizer = Arc::new(LocalWhisperTranscriber::new("base")?);

    let processor = Arc::new(LocalProcessor {
        model: "gpt2".to_string(),
    });

    let server = TcpServer::new(8080, recorder, recognizer, processor);

    let result = server.run().await?;
    println!("{}", result);
    Ok(())
}
