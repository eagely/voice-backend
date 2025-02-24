use super::tts_service::TtsService;
use crate::error::{Error::ApiError, Result};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use serde_json::json;
use url::Url;

pub struct PiperClient {
    client: Client,
    base_url: Url,
    voice: String,
}

impl PiperClient {
    pub fn new(base_url: &str, voice: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            base_url: Url::parse(base_url)?,
            voice: voice.into(),
        })
    }
}

#[async_trait]
impl TtsService for PiperClient {
    async fn synthesize(&self, text: &str) -> Result<Bytes> {
        let url = self.base_url.join("api/tts")?;

        let request_body = json!({
            "text": text,
            "voice": self.voice,
        });

        let response = self.client.post(url).json(&request_body).send().await?;

        if response.status().is_success() {
            Ok(response.bytes().await?)
        } else {
            Err(ApiError(format!(
                "Failed to synthesize speech: {}",
                response.status()
            )))
        }
    }
}
