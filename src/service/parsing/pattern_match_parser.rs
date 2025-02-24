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
