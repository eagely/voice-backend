use super::ProcessingService;
use crate::error::Result;
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::weather::WeatherService;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PatternMatchProcessor {
    weather_service: Arc<dyn WeatherService>,
    geocoding_service: Arc<dyn GeocodingService>,
    llm_service: Arc<dyn LlmService>,
}

impl PatternMatchProcessor {
    pub fn new(
        weather_service: Arc<dyn WeatherService>,
        geocoding_service: Arc<dyn GeocodingService>,
        llm_service: Arc<dyn LlmService>,
    ) -> Self {
        Self {
            weather_service,
            geocoding_service,
            llm_service,
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
impl ProcessingService for PatternMatchProcessor {
    async fn process(&self, input: &str) -> Result<String> {
        Ok(match input.to_lowercase() {
            x if x.contains("weather") || x.contains("whether") => {
                let coordinate = self
                    .geocoding_service
                    .request(&Self::remove(
                        x,
                        &["weather in", "weather", "whether in", "whether"],
                    ))
                    .await?;
                self.weather_service.request(coordinate).await?
            }
            _ => {
                let res = self.llm_service.request(input).await?;
                res
            },
        })
    }
}
