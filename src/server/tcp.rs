use crate::error::Result;
use crate::model::command::Command;
use crate::service::runtime::RuntimeService;
use crate::service::{
    parsing::ParsingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;

pub struct TcpServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingService>,
    transcriber: Box<dyn TranscriptionService>,
    parser: Box<dyn ParsingService>,
    runtime: Box<dyn RuntimeService>,
}

impl TcpServer {
    pub fn new(
        addr: &str,
        recorder: Box<dyn RecordingService>,
        transcriber: Box<dyn TranscriptionService>,
        parser: Box<dyn ParsingService>,
        runtime: Box<dyn RuntimeService>,
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
            match line.as_str().into() {
                Command::StartRecording => {
                    self.recorder.start()?;
                    recording_active = true;
                    writeln!(writer, "Recording started.")?;
                }
                Command::StopRecording => {
                    if recording_active {
                        let audio = self.recorder.stop()?;
                        recording_active = false;
                        let transcription = self.transcriber.transcribe(&audio).await?;
                        let action = self.parser.parse(&transcription).await?;
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
                    } else {
                        writeln!(writer, "No recording in progress.")?;
                    }
                }
                Command::Unknown(command) => {
                    writeln!(writer, "Unknown command: {}", command)?;
                }
            }
            line.clear();
        }

        Ok(())
    }
}
