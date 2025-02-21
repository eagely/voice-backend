mod config;
mod error;
mod model;
mod server;
mod service;

use crate::error::Result;
use server::tcp::TcpServer;
use service::{
    geocoding::NominatimClient, llm::OllamaClient, parsing::RasaClient, recording::LocalRecorder,
    runtime::LocalRuntime, transcription::LocalWhisperTranscriber, tts::PiperClient,
    weather::OpenWeatherMapClient,
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

    let parsing_service = Box::new(RasaClient::new(&config.rasa.base_url)?);

    let runtime_service = Box::new(LocalRuntime::new(
        geocoding_client,
        ollama_client,
        weather_client,
    ));

    let tts_service = Box::new(PiperClient::new(&config.tts.base_url, &config.tts.voice)?);

    let server = TcpServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        parsing_service,
        runtime_service,
        tts_service,
        Arc::new(config.response.response_type.clone()),
    )?;

    loop {
        server.listen().await?;
    }
}