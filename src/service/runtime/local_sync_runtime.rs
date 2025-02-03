use super::runtime_service::RuntimeService;
use crate::error::Result;
use crate::model::action::Action;

pub struct LocalSyncRuntime;

impl LocalSyncRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeService for LocalSyncRuntime {
    fn run(&self, action: Action) -> Result<Option<String>> {
        match action.intent.name {
            _ => Ok(Some("Could you repeat that?".to_owned())),
        }
    }
}
