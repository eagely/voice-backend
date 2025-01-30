use super::processing_service::ProcessingService;
use crate::error::Result;
use async_trait::async_trait;

pub struct LocalProcessor {
    pub model: String,
}

#[async_trait]
impl ProcessingService for LocalProcessor {
    async fn process(&self, input: &str) -> Result<String> {
        todo!()
    }
}
