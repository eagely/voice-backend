use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeocodingImplementation {
    Nominatim,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmImplementation {
    DeepSeek,
    Ollama,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParsingImplementation {
    PatternMatch,
    Rasa,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingImplementation {
    Local,
    Remote,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionImplementation {
    Deepgram,
    Local,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SynthesisImplementation {
    Elevenlabs,
    Piper,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeatherImplementation {
    OpenWeatherMap,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseKind {
    Audio,
    Text,
}
