use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Action {
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub text: String,
}

impl Action {
    pub fn new(intent: Intent, entities: Vec<Entity>, text: impl Into<String>) -> Action {
        Action {
            intent,
            entities,
            text: text.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Intent {
    pub name: IntentKind,
    pub confidence: Option<f32>,
}

impl Intent {
    pub fn new(name: IntentKind, confidence: Option<f32>) -> Intent {
        Intent { name, confidence }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentKind {
    #[serde(rename = "nlu_fallback")]
    LlmQuery,
    SetTimer,
    WeatherQuery,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub entity: String,
    pub value: EntityValue,
    #[serde(rename = "confidence_entity")]
    pub confidence: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EntityValue {
    String(String),
    Duration(DurationValue),
}

#[derive(Debug, Deserialize)]
pub struct DurationValue {
    pub value: u64,
    pub unit: String,
}

impl Entity {
    pub fn new(entity: impl Into<String>, value: EntityValue, confidence: Option<f32>) -> Entity {
        Entity {
            entity: entity.into(),
            value,
            confidence,
        }
    }

    pub fn get_duration(&self) -> Option<Duration> {
        match &self.value {
            EntityValue::Duration(d) => Some(Duration::from_secs(d.value)),
            _ => None,
        }
    }
}
