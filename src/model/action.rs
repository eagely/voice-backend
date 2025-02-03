use std::collections::HashMap;

pub struct Action {
    pub intent: Intent,
    pub entities: HashMap<String, Entity>,
}

pub struct Intent {
    pub name: String,
    pub confidence: f32,
}

pub struct Entity {
    pub name: String,
    pub confidence: f32,
}
