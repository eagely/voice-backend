use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::{Message, Utf8Bytes}, MaybeTlsStream, WebSocketStream};

use crate::{
    error::{Error, Result},
    model::command::Command,
    service::recording::RecordingService,
};

pub struct RemoteRecorder {
    ws_stream: Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl RemoteRecorder {
    pub async fn new(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url).await?;
        Ok(Self {
            ws_stream: Mutex::new(ws_stream),
        })
    }
}

#[async_trait]
impl RecordingService for RemoteRecorder {
    async fn start(&self) -> Result<()> {
        let start_message = Command::StartRecording.into();
        let mut stream = self.ws_stream.lock().await;
        stream.send(Message::Text(start_message)).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<Bytes> {
        let stop_message = Command::StopRecording.into();
        let mut stream = self.ws_stream.lock().await;
        stream.send(Message::Text(stop_message)).await?;

        if let Some(msg) = stream.next().await {
            let msg = msg?;
            match msg {
                Message::Binary(data) => Ok(Bytes::from(data)),
                Message::Text(text) => Ok(Bytes::from(Utf8Bytes::from(text))),
                _ => Err(Error::ApiError("Unexpected message type".to_string())),
            }
        } else {
            Err(Error::ApiError(
                "Failed to receive audio data".to_string(),
            ))
        }
    }
}
