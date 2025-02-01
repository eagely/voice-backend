use crate::error::{Error, Result};
use crate::service::{
    processing::ProcessingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt; // to use stream.next().await

pub struct TcpServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingService>,
    transcriber: Arc<dyn TranscriptionService + Send + Sync>,
    processor: Arc<dyn ProcessingService + Send + Sync>,
}

impl TcpServer {
    pub fn new(
        addr: &str,
        recorder: Box<dyn RecordingService>,
        transcriber: Arc<dyn TranscriptionService + Send + Sync>,
        processor: Arc<dyn ProcessingService + Send + Sync>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            listener,
            recorder,
            transcriber,
            processor,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        let (stream, _addr) = self.listener.accept()?;
        self.handle_client(stream).await?;
        Ok(())
    }

    async fn handle_client(&self, stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut writer = &stream;
        let mut line = String::new();
        let mut recording_active = false;

        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if trimmed == "START_RECORDING" {
                self.recorder.start()?;
                recording_active = true;
                writeln!(writer, "Recording started.")?;
            } else if trimmed == "STOP_RECORDING" && recording_active {
                let audio = self.recorder.stop()?;
                recording_active = false;
                let transcription = self.transcriber.transcribe(&audio).await?;
                match self.processor.process(&transcription).await {
                    Ok(mut stream) => {
                        while let Some(item) = stream.next().await {
                            let output = item?;
                            writeln!(writer, "{}", output)?;
                            writer.flush()?;
                        }
                    }
                    Err(e) => {
                        writeln!(writer, "Error processing transcription: {}", e)?;
                        writer.flush()?;
                    }
                }
            } else if trimmed == "STOP_RECORDING" {
                writeln!(writer, "No recording in progress.")?;
            } else {
                writeln!(writer, "Unknown command.")?;
            }
            line.clear();
        }

        Ok(())
    }
}
