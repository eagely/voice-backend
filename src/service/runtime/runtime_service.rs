use crate::error::Result;
use crate::model::action::Action;
use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

#[async_trait]
pub trait RuntimeService: Send + Sync {
    async fn run(
        &self,
        action: Action,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
