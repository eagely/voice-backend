use crate::config::{enums::ResponseKind, AppConfig};
use crate::error::Result;
use crate::model::command::Command;
use crate::service::runtime::RuntimeService;
use crate::service::synthesis::SynthesizerService;
use crate::service::{
    parsing::ParsingService, recording::RecordingService, transcription::TranscriptionService,
};
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

pub struct WsServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingService>,
    transcriber: Box<dyn TranscriptionService>,
    parser: Box<dyn ParsingService>,
    runtime: Box<dyn RuntimeService>,
    synthesizer: Box<dyn SynthesizerService>,
    response_kind: ResponseKind,
}

impl WsServer {
    pub async fn new(
        addr: &str,
        recorder: Box<dyn RecordingService>,
        transcriber: Box<dyn TranscriptionService>,
        parser: Box<dyn ParsingService>,
        runtime: Box<dyn RuntimeService>,
        synthesizer: Box<dyn SynthesizerService>,
        response_kind: ResponseKind,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            recorder,
            transcriber,
            parser,
            runtime,
            synthesizer,
            response_kind,
        })
    }

    async fn send_text(
        &self,
        ws_stream: &mut WebSocketStream<TcpStream>,
        text: &str,
    ) -> Result<()> {
        ws_stream
            .send(Message::Text(format!("T{}", text).into()))
            .await?;
        Ok(())
    }

    async fn send_config(
        &self,
        ws_stream: &mut WebSocketStream<TcpStream>,
        entry: &str,
    ) -> Result<()> {
        ws_stream
            .send(Message::Text(format!("C{}", entry).into()))
            .await?;
        Ok(())
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
                let cmd: Command = line.as_str().into();
                info!("Received command from client: {:?}", &cmd);
                match cmd {
                    Command::StartRecording => {
                        self.recorder.start().await?;
                        recording_active = true;
                    }
                    Command::StopRecording => {
                        let audio = self.recorder.stop().await?;
                        recording_active = false;
                        info!("Recording stopped");
                        let transcription = self.transcriber.transcribe(&audio).await?;
                        info!("Transcribed text: {:?}", &transcription);
                        let action = self.parser.parse(&transcription).await?;
                        info!("Action to perform: {:?}", &action);
                        let mut output_stream = self.runtime.run(action).await?;
                        info!("Runtime finished");
                        match &self.response_kind {
                            ResponseKind::Text => {
                                while let Some(text) = output_stream.next().await {
                                    let text = text?;
                                    info!("Sending T{:?}", text);
                                    self.send_text(&mut ws_stream, &format!("{}", text)).await?;
                                }
                            }
                            ResponseKind::Audio => {
                                let mut audio_stream =
                                    self.synthesizer.synthesize(output_stream).await?;
                                info!("Sending audio");
                                let mut audio_buffer = BytesMut::new();
                                while let Some(chunk) = audio_stream.next().await {
                                    audio_buffer.extend_from_slice(&chunk?);
                                }
                                ws_stream.send(Message::Binary(audio_buffer.into())).await?;
                                info!("Audio sent");
                            }
                        }
                    }
                    Command::Cancel => {
                        if recording_active {
                            let _ = self.recorder.stop().await?;
                            recording_active = false;
                            self.send_text(&mut ws_stream, "Recording canceled.")
                                .await?;
                        } else {
                            self.send_text(&mut ws_stream, "Nothing to cancel.").await?;
                        }
                    }
                    Command::GetConfig => {
                        let entries = AppConfig::get_all_config_entries().await?;
                        for entry in entries {
                            self.send_config(&mut ws_stream, &entry).await?;
                        }
                    }
                    Command::SetConfig(config_str) => {
                        if let Some((table_key, value)) = config_str.split_once('=') {
                            let value = value.trim();
                            if let Some((table, key)) = table_key.split_once('.') {
                                let table = table.trim();
                                let key = key.trim();
                                let value = value.trim();
                                match AppConfig::write_config(table, key, value).await {
                                    Ok(_) => {
                                        info!("Set {}.{} to {}", table, key, value);
                                        self.send_text(&mut ws_stream, "Configuration updated.")
                                            .await?;
                                        self.send_config(&mut ws_stream, format!("{}.{}={}", table, key, value).as_str()).await?;
                                    }
                                    Err(e) => {
                                        self.send_text(
                                            &mut ws_stream,
                                            &format!("Error updating configuration: {}", e),
                                        )
                                        .await?;
                                    }
                                }
                            } else {
                                self.send_text(
                                    &mut ws_stream,
                                    "Invalid format. Use table.key=value.",
                                )
                                .await?;
                            }
                        } else {
                            self.send_text(&mut ws_stream, "Invalid format. Use table.key=value.")
                                .await?;
                        }
                    }
                    Command::Unknown(command) => {
                        self.send_text(&mut ws_stream, &format!("Unknown command: {}", command))
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
