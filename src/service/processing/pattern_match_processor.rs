use super::ProcessingService;
use crate::error::Result;
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::weather::WeatherService;
use async_trait::async_trait;
use std::{pin::Pin, sync::Arc};
use tokio_stream::{once, Stream};

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
    async fn process(
        &self,
        input: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let input_lower = input.to_lowercase();

        let stream = match input_lower.as_str() {
            x if x.contains("weather") || x.contains("whether") => {
                let location = Self::remove(
                    x.to_string(),
                    &["weather in", "weather", "whether in", "whether"],
                );
                let coordinates = self.geocoding_service.request(&location).await?;
                let weather_info = self.weather_service.request(coordinates).await?;

                Box::pin(once(Ok(weather_info)))
            }
            _ => self.llm_service.request(input).await?,
        };

        Ok(stream)
    }
}
