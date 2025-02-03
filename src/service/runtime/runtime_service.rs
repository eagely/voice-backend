use crate::error::Result;
use crate::model::action::Action;

pub trait RuntimeService {
    fn run(&self, action: Action) -> Result<Option<String>>;
}
