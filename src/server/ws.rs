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
                let l: Command = line.as_str().into();
                match l {
                    Command::StartRecording => {
                        self.recorder.start().await?;
                        recording_active = true;
                    }
                    Command::StopRecording => {
                        if recording_active {
                            let audio = self.recorder.stop().await?;
                            recording_active = false;
                            println!("Recording stopped");
                            let transcription = self.transcriber.transcribe(&audio).await?;
                            println!("Transcribed text: {:?}", &transcription);
                            let action = self.parser.parse(&transcription).await?;
                            println!("Action to perform: {:?}", &action);
                            let mut output_stream = self.runtime.run(action).await?;
                            println!("Runtime finished");
                            match &self.response_kind {
                                ResponseKind::Text => {
                                    while let Some(text) = output_stream.next().await {
                                        println!("Sending {:?}", text);
                                        ws_stream.send(Message::Text(text?.into())).await?;
                                    }
                                }
                                ResponseKind::Audio => {
                                    let mut audio_stream =
                                        self.synthesizer.synthesize(output_stream).await?;
                                    println!("Sending audio");
                                    let mut audio_buffer = BytesMut::new();
                                    while let Some(audio) = audio_stream.next().await {
                                        audio_buffer.extend_from_slice(&audio?);
                                    }
                                    ws_stream.send(Message::Binary(audio_buffer.into())).await?;
                                    println!("Audio sent");
                                }
                            }
                        } else {
                            ws_stream.send("No recording in progress.".into()).await?;
                        }
                    }
                    Command::Cancel => {
                        if recording_active {
                            let _ = self.recorder.stop().await?;
                            recording_active = false;
                            ws_stream.send("Recording canceled.".into()).await?;
                        } else {
                            ws_stream.send("Nothing to cancel.".into()).await?;
                        }
                    }
                    Command::Config(config_str) => {
                        if let Some((table_key, value)) = config_str.split_once('=') {
                            let table_key = table_key.trim();
                            let value = value.trim();

                            if let Some((table, key)) = table_key.split_once('.') {
                                match AppConfig::write_config(table.trim(), key.trim(), value).await
                                {
                                    Ok(_) => {
                                        ws_stream.send("Configuration updated.".into()).await?;
                                    }
                                    Err(e) => {
                                        ws_stream
                                            .send(
                                                format!("Error updating configuration: {}", e)
                                                    .into(),
                                            )
                                            .await?;
                                    }
                                }
                            } else {
                                ws_stream
                                    .send("Invalid format. Use table.key = value.".into())
                                    .await?;
                            }
                        } else {
                            ws_stream
                                .send("Invalid format. Use table.key = value.".into())
                                .await?;
                        }
                    }
                    Command::Unknown(command) => {
                        ws_stream
                            .send(format!("Unknown command: {}", command).into())
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
