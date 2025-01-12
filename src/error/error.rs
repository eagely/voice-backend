use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error during recording: {0}")]
    IOError(#[from] tokio::io::Error),
    #[error("Devices Error: {0}")]
    DevicesError(#[from] cpal::DevicesError),
    #[error("No input device.")]
    NoDefaultInputDevice,
    #[error("Input device with the name {0} not found.")]
    InputDeviceNotFound(String),
    #[error("Failed to lock: {0}")]
    LockError(String),
    #[error("Failed to play stream: {0}")]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error("Failed to pause stream: {0}")]
    PauseStreamError(#[from] cpal::PauseStreamError),
    #[error("Request error: {0}")]
    StreamBuildError(#[from] cpal::BuildStreamError),
    #[error("Failed to create WAV writer: {0}")]
    WavWriterError(#[from] hound::Error),
}
