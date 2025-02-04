use super::processing_service::ParsingService;
use crate::{error::Result, model::action::Action};
use async_trait::async_trait;
use reqwest::Client;
use url::Url;

pub struct RasaClient {
    client: Client,
    base_url: Url,
}

impl RasaClient {
    fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            base_url: Url::parse(base_url)?,
        })
    }
}

#[async_trait]
impl ParsingService for RasaClient {
    async fn parse(&self, input: &str) -> Result<Option<Action>> {
        todo!()
    }
}
