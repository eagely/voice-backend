use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait WorkspaceService: Send + Sync {
    async fn minimize_window(&self) -> Result<()>;
    async fn maximize_window(&self) -> Result<()>;
    async fn switch_workspace(&self, workspace: usize) -> Result<()>;
    async fn close_window(&self) -> Result<()>;
    async fn open_application(&self, application: &str) -> Result<()>;
}
