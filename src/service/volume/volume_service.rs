use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait VolumeService: Send + Sync {
    async fn decrease(&self, value: u8) -> Result<()>;
    async fn increase(&self, value: u8) -> Result<()>;
    async fn set(&self, value: u8) -> Result<()>;
}
