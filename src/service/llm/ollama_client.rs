use super::LlmService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
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
    async fn request(&self, input: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": input,
            "stream": false,
        });

        let url = self.base_url.join("/api/generate")?;
        let response = self.client
            .post(url)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let output = response_json["response"]
                .as_str()
                .ok_or_else(|| Error::ApiError("Invalid response format".to_string()))?
                .to_string();
            Ok(output)
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
