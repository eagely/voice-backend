mod config;
mod error;
mod model;
mod server;
mod service;

use crate::error::Result;
use config::AppConfig;
use server::ws::WsServer;
use service::{
    geocoding::NominatimClient, llm::OllamaClient, parsing::RasaClient, recording::LocalRecorder,
    runtime::LocalRuntime, timer::memory_timer::MemoryTimer,
    transcription::LocalWhisperTranscriber, tts::PiperClient, weather::OpenWeatherMapClient,
    workspace::KWinClient,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::new()?);

    let recorder = Box::new(LocalRecorder::new(&config.recorder.device_name)?);

    let transcriber = Box::new(LocalWhisperTranscriber::new(
        &config.transcriber.model_path,
        config.transcriber.use_gpu,
    )?);

    let geocoding_service = Arc::new(NominatimClient::new(&config.geocoding.base_url)?);

    let llm_service = Arc::new(OllamaClient::new(&config.llm.model, &config.llm.base_url)?);

    let weather_service = Arc::new(OpenWeatherMapClient::new(
        std::env::var("OPENWEATHERMAP_API_KEY")?,
        &config.weather.base_url,
    )?);

    let timer_service = Arc::new(MemoryTimer::new());

    let parsing_service = Box::new(RasaClient::new(&config.rasa.base_url)?);

    let workspace_service = Arc::new(KWinClient);

    let runtime_service = Box::new(LocalRuntime::new(
        geocoding_service,
        llm_service,
        weather_service,
        timer_service,
        workspace_service,
    ));

    let tts_service = Box::new(PiperClient::new(&config.tts.base_url, &config.tts.voice)?);

    let server = WsServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        parsing_service,
        runtime_service,
        tts_service,
        Arc::new(config.response.response_type.clone()),
        config.clone(),
    )
    .await?;

    server.listen().await?;

    Ok(())
}
