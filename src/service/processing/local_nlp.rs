use std::pin::Pin;

use super::processing_service::ProcessingService;
use crate::error::Result;
use async_trait::async_trait;
use tokio_stream::Stream;

pub struct LocalProcessor {
    pub model: String,
}

#[async_trait]
impl ProcessingService for LocalProcessor {
    async fn process(
        &self,
        input: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        todo!()
    }
}
