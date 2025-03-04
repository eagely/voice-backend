use super::workspace_service::WorkspaceService;
use crate::error::{Error::WmctrlError, Result};
use async_trait::async_trait;
use std::process::Command;

pub struct KWinClient;

#[async_trait]
impl WorkspaceService for KWinClient {
    async fn minimize_window(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Window Minimize")
            .output()
            .map_err(|e| WmctrlError(e.to_string()))?;

        if !output.status.success() {
            return Err(WmctrlError(format!(
                "Failed to minimize window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn maximize_window(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Window Maximize")
            .output()
            .map_err(|e| WmctrlError(e.to_string()))?;

        if !output.status.success() {
            return Err(WmctrlError(format!(
                "Failed to maximize window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn switch_workspace(&self, workspace: usize) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.KWin")
            .arg("/KWin")
            .arg("org.kde.KWin.setCurrentDesktop")
            .arg(workspace.to_string())
            .output()
            .map_err(|e| WmctrlError(e.to_string()))?;

        if !output.status.success() {
            return Err(WmctrlError(format!(
                "Failed to switch workspace: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn close_window(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Window Close")
            .output()
            .map_err(|e| WmctrlError(e.to_string()))?;

        if !output.status.success() {
            return Err(WmctrlError(format!(
                "Failed to close window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

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
    async fn test_switch_workspace() -> Result<()> {
        let kwin_client = KWinClient;

        let switch_workspace_result = kwin_client.switch_workspace(2).await;
        assert!(
            switch_workspace_result.is_ok(),
            "Failed to switch workspace"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_close_window() -> Result<()> {
        let kwin_client = KWinClient;

        let close_window_result = kwin_client.close_window().await;
        assert!(close_window_result.is_ok(), "Failed to close window");

        Ok(())
    }
}
