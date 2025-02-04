mod error;
mod model;
mod service;
mod tcp;

use crate::error::Result;
use service::{
    geocoding::NominatimClient, llm::OllamaClient, parsing::PatternMatchParser,
    recording::LocalRecorder, runtime::local_runtime::LocalRuntime,
    transcription::LocalWhisperTranscriber, weather::OpenWeatherMapClient,
};
use std::sync::Arc;
use tcp::server::TcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let recorder = Box::new(LocalRecorder::new("pipewire")?);

    let recognizer = Arc::new(LocalWhisperTranscriber::new("base.bin")?);

    let geocoding_client = Arc::new(NominatimClient::new(
        "https://nominatim.openstreetmap.org/search",
    )?);

    let ollama_client = Arc::new(OllamaClient::new(
        "deepseek-r1:7b",
        "http://localhost:11434",
    )?);

    let weather_client = Arc::new(OpenWeatherMapClient::new(
        std::env::var("OPENWEATHERMAP_API_KEY")?,
        "https://api.openweathermap.org/data/3.0/onecall",
    )?);

    let parser = Arc::new(PatternMatchParser::new());

    let runtime = Arc::new(LocalRuntime::new(
        geocoding_client,
        ollama_client,
        weather_client,
    ));

    let server = TcpServer::new("127.0.0.1:8080", recorder, recognizer, parser, runtime)?;
    loop {
        server.listen().await?;
    }
}
