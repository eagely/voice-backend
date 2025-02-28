use crate::error::Result;
use crate::model::action::Action;
use async_trait::async_trait;
use futures::stream::BoxStream;

#[async_trait]
pub trait RuntimeService: Send + Sync {
    async fn run(
        &self,
        action: Action,
    ) -> Result<BoxStream<'static, Result<String>>>;
}
