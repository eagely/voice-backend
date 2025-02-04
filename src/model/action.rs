use std::collections::HashMap;

pub struct Action {
    pub intent: Intent,
    pub entities: Option<HashMap<String, Entity>>,
    pub text: String,
}

pub struct Intent {
    pub name: IntentKind,
    pub confidence: f32,
}

pub enum IntentKind {
    LLMQuery,
    Weather,
}

pub struct Entity {
    pub name: String,
    pub confidence: f32,
}
