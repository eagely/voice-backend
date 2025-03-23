use super::transcription_service::TranscriptionService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use serde_json::Value;
use url::Url;

pub struct DeepgramClient {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl DeepgramClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let api_key = std::env::var("DEEPGRAM_API_KEY")?;
        Ok(Self {
            client: Client::builder().build()?,
            api_key,
            base_url: Url::parse(base_url)?,
        })
    }
}

#[async_trait]
impl TranscriptionService for DeepgramClient {
    async fn transcribe(&self, audio: &Bytes) -> Result<String> {
        let mut url = self.base_url.join("listen")?;
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("model", "nova-2");
            query_pairs.append_pair("smart_format", "true");
        }

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Content-Type", "audio/wav")
            .body(audio.clone())
            .send()
            .await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;
            let transcript = json
                .get("results")
                .and_then(|r| r.get("channels"))
                .and_then(|channels| channels.get(0))
                .and_then(|channel| channel.get("alternatives"))
                .and_then(|alternatives| alternatives.get(0))
                .and_then(|alternative| alternative.get("transcript"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());

            match transcript {
                Some(text) => Ok(text),
                None => Err(Error::ApiError("No transcription result found".to_string())),
            }
        } else {
            let error_json: Value = response.json().await?;
            Err(Error::ApiError(
                error_json["err_msg"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ))
        }
    }
}
