use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait WorkspaceService: Send + Sync {
    async fn close_window(&self) -> Result<()>;
    async fn minimize_window(&self) -> Result<()>;
    async fn maximize_window(&self) -> Result<()>;
    async fn show_desktop(&self) -> Result<()>;
    async fn switch_workspace(&self, workspace: usize) -> Result<()>;
}
