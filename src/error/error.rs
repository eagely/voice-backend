use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Codec error: {0}")]
    AudioCodec(String),
    #[error("Input device with the name {0} not found.")]
    AudioInputDeviceNotFound(String),
    #[error("Devices Error: {0}")]
    AudioInputDevices(#[from] cpal::DevicesError),
    #[error("Audio Processing Error: {0}")]
    AudioProcessing(#[from] hound::Error),
    #[error("Request error: {0}")]
    AudioStreamBuild(#[from] cpal::BuildStreamError),
    #[error("Failed to load config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("Not a valid location: {0}")]
    GeocodingError(String),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("IO error during recording: {0}")]
    IoError(#[from] tokio::io::Error),
    #[error("Json deserialization error: {0}")]
    JsonDeserializationError(#[from] serde_json::Error),
    #[error("Failed to lock: {0}")]
    Lock(String),
    #[error("Notification error: {0}")]
    NotificationError(#[from] notify_rust::error::Error),
    #[error("Failed to pause stream: {0}")]
    PauseAudioStream(#[from] cpal::PauseStreamError),
    #[error("Failed to play stream: {0}")]
    PlayAudioStream(#[from] cpal::PlayStreamError),
    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("WebSocket communication error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Whisper error: {0}")]
    WhisperError(#[from] whisper_rs::WhisperError),
    #[error("Workspace management error: {0}")]
    WorkspaceManagementError(String),
}
