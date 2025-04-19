use tokio_tungstenite::tungstenite::Utf8Bytes;

#[derive(Debug)]
pub enum Command {
    Cancel,
    StartRecording,
    StopRecording,
    GetConfig,
    SetConfig(String),
    Unknown(String),
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s.trim() {
            "AC" => Self::Cancel,
            "AI" => Self::StartRecording,
            "AT" => Self::StopRecording,
            "G" => Self::GetConfig,
            x if x.starts_with('C') => Self::SetConfig(x.strip_prefix('C').unwrap().to_owned()),
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl From<Command> for String {
    fn from(command: Command) -> Self {
        match command {
            Command::Cancel => "AC".to_string(),
            Command::StartRecording => "AI".to_string(),
            Command::StopRecording => "AT".to_string(),
            Command::GetConfig => "G".to_string(),
            Command::SetConfig(s) => format!("C{}", s),
            Command::Unknown(s) => s,
        }
    }
}

impl From<Command> for Utf8Bytes {
    fn from(cmd: Command) -> Self {
        let s: String = cmd.into();
        Utf8Bytes::from(s)
    }
}
