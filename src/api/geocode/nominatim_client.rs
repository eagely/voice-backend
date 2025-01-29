use crate::{error::Result, model::geocode::GeocodeResponse};
use async_trait::async_trait;
use reqwest::{header, Client};
use url::Url;

use super::geocoding_client::GeocodingClient;

pub struct NominatimClient {
    client: Client,
    base_url: Url,
}

impl NominatimClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str("eagely's Voice Assistant/1.0")?,
        );
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            base_url: Url::parse(&base_url.into())?,
        })
    }
}

#[async_trait]
impl GeocodingClient for NominatimClient {
    async fn request(&self, address: &str) -> Result<GeocodeResponse> {
        let mut url = self.base_url.clone();
        url.query_pairs_mut()
            .append_pair("q", address)
            .append_pair("format", "json")
            .append_pair("limit", "1");
        let response = self.client.get(url).send().await?.text().await?;
        let coordinates: Vec<GeocodeResponse> = serde_json::from_str(&response)?;
        coordinates
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::GeocodingError(format!("No results for address: {}", address)))
    }
}
