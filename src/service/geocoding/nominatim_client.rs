use super::geocoding_service::GeocodingService;
use crate::{error::Result, model::geocode::GeocodeResponse};
use async_trait::async_trait;
use reqwest::{header, Client};
use url::Url;

pub struct NominatimClient {
    client: Client,
    base_url: Url,
}

impl NominatimClient {
    pub fn new(base_url: impl Into<String>, user_agent: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent)?,
        );
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            base_url: Url::parse(&base_url.into())?,
        })
    }
}

#[async_trait]
impl GeocodingService for NominatimClient {
    async fn request(&self, address: &str) -> Result<GeocodeResponse> {
        let mut url = self.base_url.clone();
        url.query_pairs_mut()
            .append_pair("q", address)
            .append_pair("format", "json")
            .append_pair("limit", "1");
        let response = self.client.get(url).send().await?.text().await?;
        let coordinates: Vec<GeocodeResponse> = serde_json::from_str(&response)?;
        coordinates.into_iter().next().ok_or_else(|| {
            crate::error::Error::GeocodingError(format!("No results for address: {}", address))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_nominatim_client() -> Result<()> {
        let config = Arc::new(AppConfig::new()?);

        let client = NominatimClient::new(&config.geocoding.base_url, &config.geocoding.user_agent)?;

        let address = "Vienna";
        let response = client.request(address).await?;

        assert_eq!(response.name, "Wien");
        assert!(!response.lat.is_empty());
        assert!(!response.lon.is_empty());

        Ok(())
    }
}
