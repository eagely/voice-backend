use super::handler::RecordingHandler;
use crate::error::Result;
use async_trait::async_trait;

pub struct LocalRecorder {
    device: String,
}

#[async_trait]
impl RecordingHandler for LocalRecorder {
    async fn start(&self) -> Result<()> {
        todo!()
    }

    async fn stop(&self) -> Result<Vec<u8>> {
        todo!()
    }
}
