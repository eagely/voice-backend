use super::synthesizer_service::SynthesizerService;
use crate::error::{
    Error::{self, ApiError},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::{
    stream::{once, BoxStream},
    StreamExt,
};
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
impl SynthesizerService for PiperClient {
    async fn synthesize(
        &self,
        text: BoxStream<'static, Result<String>>,
    ) -> Result<BoxStream<'static, Result<Bytes>>> {
        let url = self.base_url.join("api/synthesizer")?;

        let request_body = json!({
            "text": text.collect::<Vec<_>>().await.into_iter().collect::<std::result::Result<String, _>>()?,
            "voice": self.voice,
        });

        let response = self.client.post(url).json(&request_body).send().await?;

        if !response.status().is_success() {
            return Err(ApiError(format!(
                "Failed to synthesize speech: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::ApiError(format!("Failed to read response bytes: {}", e)))?;

        Ok(Box::pin(once(async move { Ok(bytes) })))
    }
}
