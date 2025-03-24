use super::workspace_service::WorkspaceService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::process::Command;

pub struct KWinClient;

impl KWinClient {
    async fn qdbus(&self, command: &str, error_message: &str) -> Result<()> {
        let output = Command::new("qdbus")
            .args(command.split(" "))
            .output()
            .map_err(|e| Error::WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(Error::WorkspaceManagementError(format!(
                "Failed to {}: {}",
                error_message,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn invoke_shortcut(&self, shortcut: &str, error_message: &str) -> Result<()> {
        self.qdbus(
            format!(
                "org.kde.kglobalaccel /component/kwin invokeShortcut {}",
                shortcut
            )
            .as_str(),
            error_message,
        )
        .await
    }
}

#[async_trait]
impl WorkspaceService for KWinClient {
    async fn close_window(&self) -> Result<()> {
        self.invoke_shortcut("Window Close", "close window").await
    }

    async fn minimize_window(&self) -> Result<()> {
        self.invoke_shortcut("Window Minimize", "minimize window")
            .await
    }

    async fn maximize_window(&self) -> Result<()> {
        self.invoke_shortcut("Window Maximize", "maximize window")
            .await
    }

    async fn show_desktop(&self) -> Result<()> {
        self.invoke_shortcut("Show Desktop", "show desktop").await
    }

    async fn switch_workspace(&self, workspace: usize) -> Result<()> {
        self.qdbus(
            format!(
                "org.kde.KWin /KWin org.kde.KWin.setCurrentDesktop {}",
                workspace
            )
            .as_str(),
            "switch workspace",
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[tokio::test]
    async fn test_close_window() -> Result<()> {
        let kwin_client = KWinClient;

        let close_window_result = kwin_client.close_window().await;
        assert!(close_window_result.is_ok(), "Failed to close window");

        Ok(())
    }

    #[tokio::test]
    async fn test_maximize_window() -> Result<()> {
        let kwin_client = KWinClient;

        let maximize_result = kwin_client.maximize_window().await;
        assert!(maximize_result.is_ok(), "Failed to maximize window");

        Ok(())
    }

    #[tokio::test]
    async fn test_minimize_window() -> Result<()> {
        let kwin_client = KWinClient;

        let minimize_result = kwin_client.minimize_window().await;
        assert!(minimize_result.is_ok(), "Failed to minimize window");

        Ok(())
    }

    #[tokio::test]
    async fn test_show_desktop() -> Result<()> {
        let kwin_client = KWinClient;

        let switch_workspace_result = kwin_client.show_desktop().await;
        assert!(switch_workspace_result.is_ok(), "Failed to show desktop");

        Ok(())
    }

    #[tokio::test]
    async fn test_switch_workspace() -> Result<()> {
        let kwin_client = KWinClient;

        let switch_workspace_result = kwin_client.switch_workspace(6).await;
        assert!(
            switch_workspace_result.is_ok(),
            "Failed to switch workspace"
        );

        Ok(())
    }
}
