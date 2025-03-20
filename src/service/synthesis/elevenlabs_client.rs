use super::SynthesizerService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use futures_util::sink::SinkExt;
use futures_util::stream::{BoxStream, SplitStream, StreamExt};
use futures_util::Stream;
use serde_json::{from_str, json, Value};
use std::env::var;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

pub struct ElevenLabsClient {
    api_key: String,
    base_url: String,
    model_id: String,
    voice_id: String,
}

impl ElevenLabsClient {
    pub fn new(
        base_url: impl Into<String>,
        model_id: impl Into<String>,
        voice_id: impl Into<String>,
    ) -> Result<Self> {
        let api_key = var("ELEVENLABS_API_KEY")?;
        Ok(Self {
            base_url: base_url.into(),
            api_key,
            voice_id: voice_id.into(),
            model_id: model_id.into(),
        })
    }

    async fn connect_websocket(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let ws_url = format!(
            "{}/v1/text-to-speech/{}/stream-input",
            self.base_url, self.voice_id
        );

        let url =
            Url::parse(&ws_url).map_err(|e| Error::ApiError(format!("Invalid URL: {}", e)))?;

        let url_str = url.as_str();

        let request = Request::builder()
            .uri(url_str)
            .header("Host", url.host_str().unwrap_or_default())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Sec-WebSocket-Version", "13")
            .header("xi-api-key", &self.api_key)
            .body(())
            .map_err(|e| Error::ApiError(format!("Failed to build request: {}", e)))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| Error::ApiError(format!("Failed to connect to WebSocket: {}", e)))?;

        Ok(ws_stream)
    }
}

struct AudioStream {
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Stream for AudioStream {
    type Item = Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(msg))) => match msg {
                Message::Text(text) => Poll::Ready(Some(
                    from_str::<Value>(&text)
                        .map_err(|e| Error::ApiError(format!("Failed to parse ElevenLabs response JSON: {}", e)))
                        .and_then(|parsed| {
                            parsed
                                .get("audio")
                                .map(|audio| audio.to_string())
                                .ok_or_else(|| {
                                    Error::ApiError(
                                        "Received message not containing audio from ElevenLabs".to_string(),
                                    )
                                })
                        }),
                )),
                Message::Close(_) => Poll::Ready(None),
                _ => Poll::Ready(Some(Err(Error::ApiError(format!(
                    "Received non-text message from ElevenLabs"
                ))))),
            },
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(Error::ApiError(format!(
                "ElevenLabs WebSocket error: {}",
                e
            ))))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[async_trait]
impl SynthesizerService for ElevenLabsClient {
    async fn synthesize(
        &self,
        text: BoxStream<'static, Result<String>>,
    ) -> Result<BoxStream<'static, Result<String>>> {
        let ws_stream = self.connect_websocket().await?;
        let audio_request = json!({
            "event": "audio_request",
            "text": text.collect::<Vec<_>>().await.into_iter().collect::<std::result::Result<String, _>>()?,
            "voice_settings": {
                "stability": 0.75,
                "similarity_boost": 1
            },
            "model_id": self.model_id,
            "output_format": "mp3_44100_128",
            "streaming": true
        });

        let (mut ws_tx, ws_rx) = ws_stream.split();

        ws_tx
            .send(Message::Text(audio_request.to_string().into()))
            .await?;

        let eos_message = json!({
            "event": "text",
            "text": ""
        });
        ws_tx
            .send(Message::Text(eos_message.to_string().into()))
            .await?;

        let stream = AudioStream { stream: ws_rx };
        Ok(Box::pin(stream))
    }
}
