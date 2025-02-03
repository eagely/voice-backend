use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Audio Processing Error: {0}")]
    AudioProcessing(#[from] hound::Error),
    #[error("Codec error: {0}")]
    AudioCodec(String),
    #[error("Devices Error: {0}")]
    AudioInputDevices(#[from] cpal::DevicesError),
    #[error("Request error: {0}")]
    AudioStreamBuild(#[from] cpal::BuildStreamError),
    #[error("Json deserialization error: {0}")]
    JsonDeserializationError(#[from] serde_json::Error),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("IO error during recording: {0}")]
    IoError(#[from] tokio::io::Error),
    #[error("Not a valid location: {0}")]
    GeocodingError(String),
    #[error("Input device with the name {0} not found.")]
    AudioInputDeviceNotFound(String),
    #[error("Failed to lock: {0}")]
    Lock(String),
    #[error("Failed to play stream: {0}")]
    PlayAudioStream(#[from] cpal::PlayStreamError),
    #[error("Failed to pause stream: {0}")]
    PauseAudioStream(#[from] cpal::PauseStreamError),
    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("Whisper error: {0}")]
    WhisperError(#[from] whisper_rs::WhisperError),
}
