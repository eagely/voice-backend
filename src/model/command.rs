#[derive(Debug)]
pub enum Command {
    Cancel,
    StartRecording,
    StopRecording,
    Config(String),
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
            x if x.starts_with('C') => Command::Config(x.strip_prefix('C').unwrap().to_owned()),
            other => Command::Unknown(other.to_string()),
        }
    }
}
