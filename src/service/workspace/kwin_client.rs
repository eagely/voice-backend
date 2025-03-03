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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[tokio::test]
    async fn test_maximize_and_minimize_window() -> Result<()> {
        let kwin_client = KWinClient;

        let maximize_result = kwin_client.maximize_window().await;
        assert!(maximize_result.is_ok(), "Failed to maximize window");

        let minimize_result = kwin_client.minimize_window().await;
        assert!(minimize_result.is_ok(), "Failed to minimize window");

        Ok(())
    }
}
