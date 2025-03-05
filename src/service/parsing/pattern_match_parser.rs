use super::ParsingService;
use crate::model::action::{DurationValue, Entity, EntityValue, Intent, IntentKind};
use crate::{error::Result, model::action::Action};
use async_trait::async_trait;
use regex::Regex;
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

    fn get_closest_number(input: &str, keyword: &str) -> Option<usize> {
        let re = Regex::new(r"\d+|one|two|three|four|five|six|seven|eight|nine|ten").unwrap();
        let closest_number: Option<usize> = None;

        let spelled_out_numbers: HashMap<&str, usize> = [
            ("one", 1),
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
            ("six", 6),
            ("seven", 7),
            ("eight", 8),
            ("nine", 9),
            ("ten", 10),
        ]
        .iter()
        .cloned()
        .collect();

        if let Some(keyword_pos) = input.find(keyword) {
            for cap in re.captures_iter(&input[keyword_pos + keyword.len()..]) {
                let number = if let Ok(num) = cap[0].parse::<usize>() {
                    num
                } else {
                    *spelled_out_numbers.get(&cap[0]).unwrap_or(&0)
                };

                if number != 0 {
                    return Some(number);
                }
            }
        }

        closest_number
    }

    fn extract_duration(input: &str) -> Option<DurationValue> {
        let re = Regex::new(r"(\d+)\s*(seconds?|minutes?|hours?)").unwrap();
        if let Some(caps) = re.captures(input) {
            let value = caps.get(1)?.as_str().parse::<u64>().ok()?;
            let unit = caps.get(2)?.as_str().to_string();
            Some(DurationValue { value, unit })
        } else {
            None
        }
    }
}

#[async_trait]
impl ParsingService for PatternMatchParser {
    async fn parse(&self, input: &str) -> Result<Action> {
        let input_lower = input.to_lowercase();

        match input_lower.as_str() {
            x if x.contains("close") => Ok(Action::new(
                Intent::new(IntentKind::CloseWindow, None),
                Vec::new(),
                input.to_string(),
            )),
            x if x.contains("minimize") => Ok(Action::new(
                Intent::new(IntentKind::MinimizeWindow, None),
                Vec::new(),
                input.to_string(),
            )),
            x if x.contains("maximize") => Ok(Action::new(
                Intent::new(IntentKind::MaximizeWindow, None),
                Vec::new(),
                input.to_string(),
            )),
            x if x.contains("open") => {
                let application = Self::remove(x.to_string(), &["open"]);
                let entities = vec![Entity::new(
                    "APPLICATION",
                    EntityValue::String(application),
                    None,
                )];
                Ok(Action::new(
                    Intent::new(IntentKind::OpenApplication, None),
                    entities,
                    input.to_string(),
                ))
            }
            x if x.contains("timer") || x.contains("alarm") => {
                if let Some(duration) = Self::extract_duration(x) {
                    let entities = vec![Entity::new(
                        "duration",
                        EntityValue::Duration(duration),
                        None,
                    )];
                    Ok(Action::new(
                        Intent::new(IntentKind::SetTimer, None),
                        entities,
                        input.to_string(),
                    ))
                } else {
                    Ok(Action::new(
                        Intent::new(IntentKind::LlmQuery, None),
                        Vec::new(),
                        "Please specify a clear duration for the timer.".to_string(),
                    ))
                }
            }
            x if x.contains("switch") && (x.contains("workspace") || x.contains("desktop")) => {
                if let Some(index) = Self::get_closest_number(x, "switch") {
                    Ok(Action::new(
                        Intent::new(IntentKind::SwitchWorkspace, None),
                        vec![Entity::new("NUMBER", EntityValue::Index(index), None)],
                        input.to_string(),
                    ))
                } else {
                    Ok(Action::new(
                        Intent::new(IntentKind::LlmQuery, None),
                        Vec::new(),
                        input.to_string(),
                    ))
                }
            }
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
    #[tokio::test]
    async fn test_pattern_match_parser_minimize_window() -> Result<()> {
        let parser = PatternMatchParser::new();

        let action = parser.parse("Minimize the window").await?;
        assert_eq!(action.intent.name, IntentKind::MinimizeWindow);
        assert!(action.entities.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_match_parser_maximize_window() -> Result<()> {
        let parser = PatternMatchParser::new();

        let action = parser.parse("Maximize the window").await?;
        assert_eq!(action.intent.name, IntentKind::MaximizeWindow);
        assert!(action.entities.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_match_parser_close_window() -> Result<()> {
        let parser = PatternMatchParser::new();

        let action = parser.parse("Close the window").await?;
        assert_eq!(action.intent.name, IntentKind::CloseWindow);
        assert!(action.entities.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_match_parser_switch_workspace() -> Result<()> {
        let parser = PatternMatchParser::new();

        let action = parser.parse("Switch to workspace 3").await?;
        assert_eq!(action.intent.name, IntentKind::SwitchWorkspace);
        assert_eq!(action.entities[0].entity, "NUMBER");
        assert_eq!(action.entities[0].value, EntityValue::Index(3));

        let action = parser.parse("Switch to workspace three").await?;
        assert_eq!(action.intent.name, IntentKind::SwitchWorkspace);
        assert_eq!(action.entities[0].entity, "NUMBER");
        assert_eq!(action.entities[0].value, EntityValue::Index(3));

        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_match_parser_llm_query() -> Result<()> {
        let parser = PatternMatchParser::new();

        let action = parser.parse("Tell me a joke").await?;
        assert_eq!(action.intent.name, IntentKind::LlmQuery);
        assert!(action.entities.is_empty());

        Ok(())
    }
}
