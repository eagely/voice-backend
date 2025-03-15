use super::LlmService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use futures_util::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde_json::{from_str, Value};
use std::{env, str::from_utf8};
use url::Url;

pub struct DeepSeekClient {
    client: Client,
    model: String,
    base_url: Url,
    bearer_token: String,
}

impl DeepSeekClient {
    pub fn new(model: impl Into<String>, base_url: &str) -> Result<Self> {
        let bearer_token = env::var("DEEPSEEK_API_KEY")?;

        Ok(Self {
            client: Client::builder().build()?,
            model: model.into(),
            base_url: Url::parse(base_url)?,
            bearer_token,
        })
    }
}

#[async_trait]
impl LlmService for DeepSeekClient {
    async fn request(&self, input: &str) -> Result<BoxStream<'static, Result<String>>> {
        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "content": "You are a helpful voice assistant that answers questions.",
                    "role": "system"
                },
                {
                    "content": input,
                    "role": "user"
                }
            ],
            "frequency_penalty": 0,
            "max_tokens": 2048,
            "presence_penalty": 0,
            "response_format": {
                "type": "text"
            },
            "stop": null,
            "stream": true,
            "stream_options": null,
            "temperature": 1,
            "top_p": 1,
            "tools": null,
            "tool_choice": "none",
            "logprobs": false,
            "top_logprobs": null
        });

        let url = self.base_url.join("/v1/chat/completions")?;

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let stream = response.bytes_stream().map(|chunk| {
                chunk.map_err(Error::from).and_then(|bytes| {
                    let chunk_str =
                        from_utf8(&bytes).map_err(|e| Error::ApiError(e.to_string()))?;

                    let events: Vec<_> = chunk_str
                        .split("\n\n")
                        .filter(|event| !event.is_empty())
                        .collect();

                    let mut result = String::new();
                    for event in events {
                        if event.starts_with("data: ") {
                            let json_str = event.trim_start_matches("data: ");
                            if json_str == "[DONE]" {
                                continue;
                            }

                            let json_value: Value = from_str(json_str)?;

                            if let Some(content) =
                                json_value["choices"][0]["delta"]["content"].as_str()
                            {
                                result.push_str(content);
                            }
                        }
                    }

                    Ok(result)
                })
            });

            Ok(stream.boxed())
        } else {
            let error_json: Value = response.json().await?;
            Err(Error::ApiError(
                error_json["error"]["message"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ))
        }
    }
}
