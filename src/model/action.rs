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
    WeatherQuery,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub entity: String,
    pub value: String,
    #[serde(rename = "confidence_entity")]
    pub confidence: Option<f32>,
}

impl Entity {
    pub fn new(
        entity: impl Into<String>,
        value: impl Into<String>,
        confidence: Option<f32>,
    ) -> Entity {
        Entity {
            entity: entity.into(),
            value: value.into(),
            confidence,
        }
    }
}
