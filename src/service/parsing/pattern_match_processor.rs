use super::ParsingService;
use crate::model::action::{Entity, Intent};
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::weather::WeatherService;
use crate::{error::Result, model::action::Action};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct PatternMatchParser {
    weather_service: Arc<dyn WeatherService>,
    geocoding_service: Arc<dyn GeocodingService>,
    llm_service: Arc<dyn LlmService>,
}

impl PatternMatchParser {
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
impl ParsingService for PatternMatchParser {
    async fn parse(&self, input: &str) -> Result<Option<Action>> {
        let input_lower = input.to_lowercase();

        match input_lower.as_str() {
            x if x.contains("weather") || x.contains("whether") => {
                let location = Self::remove(
                    x.to_string(),
                    &["weather in", "weather", "whether in", "whether"],
                );
                let mut entities = HashMap::new();
                entities.insert(
                    "location".to_owned(),
                    Entity {
                        name: location,
                        confidence: 100f32,
                    },
                );
                Ok(Some(Action {
                    intent: Intent {
                        name: "weather".to_owned(),
                        confidence: 100f32,
                    },
                    entities,
                }))
            }
            _ => Ok(None),
        }
    }
}
