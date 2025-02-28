use crate::error::Result;
use bytes::Bytes;

pub trait RecordingService {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<Bytes>;
}
