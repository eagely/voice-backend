pub enum Command {
    Cancel,
    StartRecording,
    StopRecording,
    Unknown(String),
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "cancel" => Self::Cancel,
            "start_recording" => Self::StartRecording,
            "stop_recording" => Self::StopRecording,
            other => Command::Unknown(other.to_string()),
        }
    }
}
