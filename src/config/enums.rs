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
pub enum ParserImplementation {
    PatternMatch,
    Rasa,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecorderImplementation {
    Local,
    Remote,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriberImplementation {
    Deepgram,
    Local,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TtsImplementation {
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
    Text,
    Audio,
}
