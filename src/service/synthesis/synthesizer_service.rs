use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;

#[async_trait]
pub trait SynthesizerService: Send + Sync {
    async fn synthesize(&self, text: BoxStream<'static, Result<String>>) -> Result<BoxStream<'static, Result<Bytes>>>;
}