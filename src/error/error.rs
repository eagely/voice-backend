use thiserror::Error;
use whisper_rs::WhisperError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Audio Processing Error: {0}")]
    AudioProcessing(#[from] hound::Error),
    #[error("Codec error: {0}")]
    AudioCodec(String),
    #[error("Devices Error: {0}")]
    AudioInputDevices(#[from] cpal::DevicesError),
    #[error("IO error during recording: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("No input device.")]
    NoDefaultAudioInputDevice,
    #[error("Input device with the name {0} not found.")]
    AudioInputDeviceNotFound(String),
    #[error("Failed to lock: {0}")]
    Lock(String),
    #[error("Failed to play stream: {0}")]
    PlayAudioStream(#[from] cpal::PlayStreamError),
    #[error("Failed to pause stream: {0}")]
    PauseAudioStream(#[from] cpal::PauseStreamError),
    #[error("Request error: {0}")]
    AudioStreamBuild(#[from] cpal::BuildStreamError),
    #[error("Whisper error: {0}")]
    WhisperError(#[from] WhisperError),
}
