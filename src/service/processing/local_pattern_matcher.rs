use super::processing_service::ProcessingService;
use crate::error::Result;
use crate::service::geocoding::GeocodingService;
use crate::service::weather::WeatherService;
use async_trait::async_trait;
use std::sync::Arc;

pub struct LocalPatternMatcher {
    weather_client: Arc<dyn WeatherService>,
    geocoding_client: Arc<dyn GeocodingService>,
}

impl LocalPatternMatcher {
    pub fn new(
        weather_client: Arc<dyn WeatherService>,
        geocoding_client: Arc<dyn GeocodingService>,
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
impl ProcessingService for LocalPatternMatcher {
    async fn process(&self, input: &str) -> Result<String> {
        Ok(match input.to_lowercase() {
            x if x.contains("weather") || x.contains("whether") => {
                let coordinate = self
                    .geocoding_client
                    .request(&Self::remove(
                        x,
                        &["weather in", "weather", "whether in", "whether"],
                    ))
                    .await?;
                self.weather_client.request(coordinate).await?
            }
            _ => input.to_owned(),
        })
    }
}
