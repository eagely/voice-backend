use std::pin::Pin;

use super::LlmService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use url::Url;

pub struct OllamaClient {
    client: Client,
    model: String,
    base_url: Url,
}

impl OllamaClient {
    pub fn new(model: impl Into<String>, base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            model: model.into(),
            base_url: Url::parse(base_url)?,
        })
    }
}

#[async_trait]
impl LlmService for OllamaClient {
    async fn request(
        &self,
        input: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": input,
        });

        let url = self.base_url.join("/api/generate")?;
        let response = self.client.post(url).json(&request_body).send().await?;

        if response.status().is_success() {
            let stream = response.bytes_stream().map(|chunk| {
                chunk.map_err(Error::from).and_then(|bytes| {
                    let json_str =
                        std::str::from_utf8(&bytes).map_err(|e| Error::ApiError(e.to_string()))?;
                    let json_value: Value = serde_json::from_str(json_str)
                        .map_err(|e| Error::JsonDeserializationError(e))?;
                    let text = json_value["response"].as_str().unwrap_or("").to_string();
                    Ok(text)
                })
            });

            Ok(Box::pin(stream))
        } else {
            let error_json: std::result::Result<Value, _> = response.json().await;
            let error_message = match error_json {
                Ok(json) => json["error"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
                Err(_) => "Unknown error".to_string(),
            };
            Err(Error::ApiError(error_message))
        }
    }
}
