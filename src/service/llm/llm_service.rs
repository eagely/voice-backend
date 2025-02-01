use std::pin::Pin;

use crate::error::Result;
use async_trait::async_trait;
use tokio_stream::Stream;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn request(
        &self,
        input: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
