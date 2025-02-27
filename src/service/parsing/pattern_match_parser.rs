use super::ParsingService;
use crate::model::action::{Entity, EntityValue, Intent, IntentKind};
use crate::{error::Result, model::action::Action};
use async_trait::async_trait;

pub struct PatternMatchParser;

impl PatternMatchParser {
    pub fn new() -> Self {
        Self
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
    async fn parse(&self, input: &str) -> Result<Action> {
        let input_lower = input.to_lowercase();

        match input_lower.as_str() {
            x if x.contains("weather") || x.contains("whether") => {
                let location = Self::remove(
                    x.to_string(),
                    &["weather in", "weather", "whether in", "whether"],
                );
                let entities = vec![Entity::new("GPE", EntityValue::String(location), None)];
                Ok(Action::new(
                    Intent::new(IntentKind::WeatherQuery, None),
                    entities,
                    input.to_string(),
                ))
            }
            _ => Ok(Action::new(
                Intent::new(IntentKind::LlmQuery, None),
                Vec::new(),
                input.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::model::action::{EntityValue, IntentKind};
    use crate::service::parsing::RasaClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_rasa_client_parse_weather_intent() -> Result<()> {
        let config = Arc::new(AppConfig::new()?);

        let rasa_client = RasaClient::new(&config.rasa.base_url)?;

        let action = rasa_client.parse("Weather in Vienna").await?;

        assert_eq!(action.intent.name, IntentKind::WeatherQuery);
        assert!(action.intent.confidence.unwrap_or(0.0) > 0.0);
        assert_eq!(action.text, "Weather in Vienna");
        assert!(!action.entities.is_empty());
        assert_eq!(action.entities[0].entity, "GPE");
        assert_eq!(
            action.entities[0].value,
            EntityValue::String("Vienna".to_string())
        );

        Ok(())
    }
}
