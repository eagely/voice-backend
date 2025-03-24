use super::VolumeService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::process::Command;

pub struct PactlClient;

impl PactlClient {
    async fn set_default_sink_volume(&self, value: &str) -> Result<()> {
        let output = Command::new("pactl")
            .arg("set-sink-volume")
            .arg("@DEFAULT_SINK@")
            .arg(value)
            .output()
            .map_err(|e| Error::VolumeAdjustmentError(e.to_string()))?;

        if !output.status.success() {
            return Err(Error::VolumeAdjustmentError(format!(
                "Failed to adjust volume: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl VolumeService for PactlClient {
    async fn decrease(&self, value: u8) -> Result<()> {
        self.set_default_sink_volume(format!("-{}%", value).as_str())
            .await
    }

    async fn increase(&self, value: u8) -> Result<()> {
        self.set_default_sink_volume(format!("+{}%", value).as_str())
            .await
    }

    async fn set(&self, value: u8) -> Result<()> {
        self.set_default_sink_volume(format!("{}%", value).as_str())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decrease_volume() {
        let client = PactlClient;
        let result = client.decrease(10).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_increase_volume() {
        let client = PactlClient;
        let result = client.increase(10).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_volume() {
        let client = PactlClient;
        let result = client.set(50).await;
        assert!(result.is_ok());
    }
}
