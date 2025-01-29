use std::sync::Arc;

use super::handler::ProcessingHandler;
use crate::{
    api::{geocode::geocoding_client::GeocodingClient, weather::WeatherClient},
    error::Result,
};
use async_trait::async_trait;

pub struct LocalPatternMatcher {
    weather_client: Arc<dyn WeatherClient>,
    geocoding_client: Arc<dyn GeocodingClient>,
}

impl LocalPatternMatcher {
    pub fn new(
        weather_client: Arc<dyn WeatherClient>,
        geocoding_client: Arc<dyn GeocodingClient>,
    ) -> Self {
        Self {
            weather_client,
            geocoding_client,
        }
    }
    
    fn remove(original: String, strings: &[&str]) -> String {
        let mut result = original;
        for &s in strings {
            result = result.replace(s, "");
        }
        result
    }
}

#[async_trait]
impl ProcessingHandler for LocalPatternMatcher {
    async fn process(&self, input: &str) -> Result<String> {
        Ok(match input.to_lowercase() {
            x if x.contains("weather") || x.contains("whether") => {
                let coordinate = self
                    .geocoding_client
                    .request(&Self::remove(x, &["weather in", "weather", "whether in", "whether"]))
                    .await?;
                self.weather_client.request(coordinate).await?
            }
            _ => input.to_owned(),
        })
    }
}
