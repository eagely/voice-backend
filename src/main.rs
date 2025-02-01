mod error;
mod model;
mod service;
mod tcp;

use crate::error::Result;
use service::{
    geocoding::NominatimClient, llm::OllamaClient, processing::PatternMatchProcessor,
    recording::LocalRecorder, transcription::LocalWhisperTranscriber,
    weather::OpenWeatherMapClient,
};
use std::sync::Arc;
use tcp::server::TcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let recorder = Box::new(LocalRecorder::new("pipewire")?);

    let recognizer = Arc::new(LocalWhisperTranscriber::new("base.bin")?);

    let weather_client = Arc::new(OpenWeatherMapClient::new(
        std::env::var("OPENWEATHERMAP_API_KEY")?,
        "https://api.openweathermap.org/data/3.0/onecall",
    )?);
    
    let geocoding_client = Arc::new(NominatimClient::new(
        "https://nominatim.openstreetmap.org/search",
    )?);
    
    let ollama_client = Arc::new(OllamaClient::new(
        "deepseek-r1:7b",
        "http://localhost:11434",
    )?);
    
    let processor = Arc::new(PatternMatchProcessor::new(
        weather_client,
        geocoding_client,
        ollama_client,
    ));

    let server = TcpServer::new("127.0.0.1:8080", recorder, recognizer, processor)?;
    loop {
        server.listen().await?;
    }
}
