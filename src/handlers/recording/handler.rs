use bytes::Bytes;

use crate::error::Result;

pub trait RecordingHandler {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<Bytes>;
}
