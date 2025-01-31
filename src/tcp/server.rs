use crate::error::{Error, Result};
use crate::service::{
    processing::ProcessingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

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
            let transcription = match line.trim() {
                "START_RECORDING" => {
                    self.recorder.start()?;
                    recording_active = true;
                    "Recording started.".to_string()
                }
                "STOP_RECORDING" if recording_active => {
                    let audio = self.recorder.stop()?;
                    recording_active = false;
                    self.transcriber.transcribe(&audio).await?
                }
                "STOP_RECORDING" => "No recording in progress.".to_string(),
                _ => "Unknown command.".to_string(),
            };

            let result = match self.processor.process(&transcription).await {
                Ok(res) => res,
                Err(e) => format!("Error processing transcription: {}", e),
            };

            if let Err(e) = writeln!(writer, "{}", result) {
                return Err(Error::ClientWriteError(format!(
                    "Failed to write to client: {}",
                    e
                )));
            }
            line.clear();
        }

        Ok(())
    }
}
