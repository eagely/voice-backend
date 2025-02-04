use super::ParsingService;
use crate::model::action::{Entity, Intent, IntentKind};
use crate::{error::Result, model::action::Action};
use async_trait::async_trait;
use std::collections::HashMap;

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
                        name: IntentKind::Weather,
                        confidence: 100f32,
                    },
                    entities: Some(entities),
                    text: input.to_owned(),
                }))
            }
            _ => Ok(Some(Action {
                intent: Intent {
                    name: IntentKind::LLMQuery,
                    confidence: 100f32,
                },
                entities: None,
                text: input.to_owned(),
            })),
        }
    }
}
