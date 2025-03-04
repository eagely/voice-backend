use std::{fmt, time::Duration};

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};

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

#[derive(Debug, PartialEq, Eq)]
pub enum IntentKind {
    CloseWindow,
    LlmQuery,
    MaximizeWindow,
    MinimizeWindow,
    SetTimer,
    SwitchWorkspace,
    WeatherQuery,
    Other(String),
}

impl<'de> Deserialize<'de> for IntentKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IntentKindVisitor;

        impl<'de> Visitor<'de> for IntentKindVisitor {
            type Value = IntentKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing an intent kind")
            }

            fn visit_str<E>(self, value: &str) -> Result<IntentKind, E>
            where
                E: Error,
            {
                match value {
                    "close_window" => Ok(IntentKind::CloseWindow),
                    "nlu_fallback" => Ok(IntentKind::LlmQuery),
                    "maximize_window" => Ok(IntentKind::MaximizeWindow),
                    "minimize_window" => Ok(IntentKind::MinimizeWindow),
                    "set_timer" => Ok(IntentKind::SetTimer),
                    "switch_workspace" => Ok(IntentKind::SwitchWorkspace),
                    "weather_query" => Ok(IntentKind::WeatherQuery),
                    _ => Ok(IntentKind::Other(value.to_owned())),
                }
            }
        }

        deserializer.deserialize_str(IntentKindVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub entity: String,
    pub value: EntityValue,
    #[serde(rename = "confidence_entity")]
    pub confidence: Option<f32>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EntityValue {
    Index(usize),
    Duration(DurationValue),
    String(String),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
