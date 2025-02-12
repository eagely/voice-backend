pub enum Command {
    StartRecording,
    StopRecording,
    Unknown(String),
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        match s.trim() {
            "START_RECORDING" => Self::StartRecording,
            "STOP_RECORDING" => Self::StopRecording,
            other => Command::Unknown(other.to_string())
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.trim() {
            "START_RECORDING" => Self::StartRecording,
            "STOP_RECORDING" => Self::StopRecording,
            other => Command::Unknown(other.to_string())
        }
    }
}