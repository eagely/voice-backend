use crate::config::ResponseType;
use crate::error::Result;
use crate::model::command::Command;
use crate::service::runtime::RuntimeService;
use crate::service::tts::TtsService;
use crate::service::{
    parsing::ParsingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use futures::{SinkExt, StreamExt};

pub struct WsServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingService>,
    transcriber: Box<dyn TranscriptionService>,
    parser: Box<dyn ParsingService>,
    runtime: Box<dyn RuntimeService>,
    tts_service: Box<dyn TtsService>,
    response_type: Arc<ResponseType>,
}

impl WsServer {
    pub async fn new(
        addr: &str,
        recorder: Box<dyn RecordingService>,
        transcriber: Box<dyn TranscriptionService>,
        parser: Box<dyn ParsingService>,
        runtime: Box<dyn RuntimeService>,
        tts: Box<dyn TtsService>,
        response_type: Arc<ResponseType>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            recorder,
            transcriber,
            parser,
            runtime,
            tts_service: tts,
            response_type,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        let (stream, _addr) = self.listener.accept().await?;
        let ws_stream = accept_async(stream).await?;
        self.handle_client(ws_stream).await?;
        Ok(())
    }

    async fn handle_client(&self, mut ws_stream: WebSocketStream<TcpStream>) -> Result<()> {
        let mut recording_active = false;

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(line) = msg {
                match line.as_str().into() {
                    Command::StartRecording => {
                        self.recorder.start()?;
                        recording_active = true;
                        ws_stream.send("Recording started.".into()).await?;
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
                                    Ok(text) => match &*self.response_type {
                                        ResponseType::Text => {
                                            ws_stream.send(text.into()).await?;
                                        }
                                        ResponseType::Audio => {
                                            let audio = self.tts_service.synthesize(&text).await?;
                                            ws_stream.send(Message::Binary(audio)).await?;
                                        }
                                    },
                                    Err(e) => {
                                        ws_stream.send(format!("Error: {}", e).into()).await?;
                                    }
                                }
                            }
                        } else {
                            ws_stream.send("No recording in progress.".into()).await?;
                        }
                    }
                    Command::Cancel => {
                        if recording_active {
                            let _ = self.recorder.stop()?;
                            recording_active = false;
                            ws_stream.send("Recording canceled.".into()).await?;
                        } else {
                            ws_stream.send("Nothing to cancel.".into()).await?;
                        }
                    }
                    Command::Unknown(command) => {
                        ws_stream.send(format!("Unknown command: {}", command).into()).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
