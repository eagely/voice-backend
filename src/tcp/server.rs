use crate::error::Result;
use crate::service::runtime::runtime_service::RuntimeService;
use crate::service::{
    parsing::ParsingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio_stream::StreamExt;

pub struct TcpServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingService>,
    transcriber: Arc<dyn TranscriptionService>,
    parser: Arc<dyn ParsingService>,
    runtime: Arc<dyn RuntimeService>,
}

impl TcpServer {
    pub fn new(
        addr: &str,
        recorder: Box<dyn RecordingService>,
        transcriber: Arc<dyn TranscriptionService>,
        parser: Arc<dyn ParsingService>,
        runtime: Arc<dyn RuntimeService>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            listener,
            recorder,
            transcriber,
            parser,
            runtime,
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
                if let Some(action) = self.parser.parse(&transcription).await? {
                    let mut output_stream = self.runtime.run(action).await?;
                    while let Some(output) = output_stream.next().await {
                        match output {
                            Ok(text) => {
                                writeln!(writer, "{}", text)?;
                            }
                            Err(e) => {
                                writeln!(writer, "Error: {}", e)?;
                            }
                        }
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
