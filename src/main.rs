mod config;
mod error;
mod model;
mod server;
mod service;

use crate::error::Result;
use server::tcp::TcpServer;
use service::{
    geocoding::NominatimClient, llm::OllamaClient, parsing::RasaClient, recording::LocalRecorder,
    runtime::LocalRuntime, transcription::LocalWhisperTranscriber, weather::OpenWeatherMapClient,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(config::AppConfig::new()?);

    let recorder = Box::new(LocalRecorder::new(&config.recorder.device_name)?);

    let transcriber = Box::new(LocalWhisperTranscriber::new(
        &config.transcriber.model_path,
        config.transcriber.use_gpu,
    )?);

    let geocoding_client = Arc::new(NominatimClient::new(&config.geocoding.base_url)?);

    let ollama_client = Arc::new(OllamaClient::new(&config.llm.model, &config.llm.base_url)?);

    let weather_client = Arc::new(OpenWeatherMapClient::new(
        std::env::var("OPENWEATHERMAP_API_KEY")?,
        &config.weather.base_url,
    )?);

    let rasa_client = Box::new(RasaClient::new(&config.rasa.base_url)?);

    let runtime = Box::new(LocalRuntime::new(
        geocoding_client,
        ollama_client,
        weather_client,
    ));

    let server = TcpServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        rasa_client,
        runtime,
    )?;

    loop {
        server.listen().await?;
    }
}
