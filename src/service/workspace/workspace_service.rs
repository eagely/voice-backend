use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait WorkspaceService: Send + Sync {
    async fn minimize_window(&self) -> Result<()>;
    async fn maximize_window(&self) -> Result<()>;
}
