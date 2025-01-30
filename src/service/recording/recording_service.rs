use bytes::Bytes;

use crate::error::Result;

pub trait RecordingService {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<Bytes>;
}
