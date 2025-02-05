use super::processing_service::ParsingService;
use crate::{
    error::Result,
    model::action::{Action, Intent, IntentKind},
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use url::Url;

pub struct RasaClient {
    client: Client,
    base_url: Url,
}

impl RasaClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            base_url: Url::parse(base_url)?,
        })
    }
}

#[async_trait]
impl ParsingService for RasaClient {
    async fn parse(&self, input: &str) -> Result<Action> {
        let url = self.base_url.join("/model/parse")?;
        let input_json = json!({ "text": input });
        let text = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&input_json)
            .send()
            .await?
            .text()
            .await?;
        let action: Action = serde_json::from_str(&text).unwrap_or(Action::new(
            Intent::new(IntentKind::LlmQuery, None),
            Vec::new(),
            input,
        ));
        Ok(action)
    }
}
