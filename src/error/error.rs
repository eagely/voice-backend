use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Audio Codec error: {0}")]
    AudioCodec(String),
    #[error("Audio input device with the name {0} not found.")]
    AudioInputDeviceNotFound(String),
    #[error("Audio input devices error: {0}")]
    AudioInputDevices(#[from] cpal::DevicesError),
    #[error("Audio processing error: {0}")]
    AudioProcessing(#[from] hound::Error),
    #[error("Audio stream build error: {0}")]
    AudioStreamBuild(#[from] cpal::BuildStreamError),
    #[error("Audio stream error: {0}")]
    AudioStreamError(#[from] cpal::StreamError),
    #[error("Config error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("Config read error: {0}")]
    ConfigReadError(#[from] toml::de::Error),
    #[error("Config write error: {0}")]
    ConfigWriteError(#[from] toml::ser::Error),
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("Geocoding error: {0}")]
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
    #[error("Pause audio stream error: {0}")]
    PauseAudioStream(#[from] cpal::PauseStreamError),
    #[error("Play audio stream error: {0}")]
    PlayAudioStream(#[from] cpal::PlayStreamError),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Volume adjustment error: {0}")]
    VolumeAdjustmentError(String),
    #[error("Wake word error: {0}")]
    WakeWordError(#[from] porcupine::PorcupineError),
    #[error("WebSocket communication error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Whisper error: {0}")]
    WhisperError(#[from] whisper_rs::WhisperError),
    #[error("Workspace management error: {0}")]
    WorkspaceManagementError(String),
}
