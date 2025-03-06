use super::workspace_service::WorkspaceService;
use crate::error::{Error::WorkspaceManagementError, Result};
use async_trait::async_trait;
use std::process::Command;

pub struct KWinClient;

#[async_trait]
impl WorkspaceService for KWinClient {
    async fn close_window(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Window Close")
            .output()
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
                "Failed to close window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn minimize_window(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Window Minimize")
            .output()
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
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
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
                "Failed to maximize window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn run_command(&self, application: &str) -> Result<()> {
        let output = Command::new(application)
            .output()
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
                "Failed to run command: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn show_desktop(&self) -> Result<()> {
        let output = Command::new("qdbus")
            .arg("org.kde.kglobalaccel")
            .arg("/component/kwin")
            .arg("invokeShortcut")
            .arg("Show Desktop")
            .output()
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
                "Failed to show desktop: {}",
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
            .map_err(|e| WorkspaceManagementError(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceManagementError(format!(
                "Failed to switch workspace: {}",
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
    async fn test_run_command() -> Result<()> {
        let kwin_client = KWinClient;

        let run_command_result = kwin_client.run_command("zeditor").await;
        assert!(
            run_command_result.is_ok(),
            "Failed to run co: zeditor"
        );

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

        let switch_workspace_result = kwin_client.switch_workspace(2).await;
        assert!(
            switch_workspace_result.is_ok(),
            "Failed to switch workspace"
        );

        Ok(())
    }
}
