use crate::{error::Result, model::action::Action};
use async_trait::async_trait;

#[async_trait]
pub trait ParsingService: Send + Sync {
    async fn parse(&self, input: &str) -> Result<Action>;
}
