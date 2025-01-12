use super::handler::ProcessingHandler;
use crate::error::Result;
use async_trait::async_trait;

pub struct LocalProcessor {
    pub model: String,
}

#[async_trait]
impl ProcessingHandler for LocalProcessor {
    async fn respond(&self, input: &str) -> Result<String> {
        todo!()
    }
}
