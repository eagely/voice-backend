mod config;
mod error;
mod model;
mod server;
mod service;

use crate::error::Result;
use config::{
    enums::{
        GeocodingImplementation, LlmImplementation, ParserImplementation, RecorderImplementation, TranscriberImplementation, TtsImplementation, WeatherImplementation
    },
    AppConfig,
};
use server::ws::WsServer;
use service::{
    geocoding::{GeocodingService, NominatimClient},
    llm::{LlmService, OllamaClient},
    parsing::{ParsingService, PatternMatchParser, RasaClient},
    recording::{LocalRecorder, RecordingService},
    runtime::LocalRuntime,
    timer::memory_timer::MemoryTimer,
    transcription::{LocalWhisperTranscriber, TranscriptionService},
    tts::{PiperClient, TtsService},
    weather::{OpenWeatherMapClient, WeatherService},
    workspace::KWinClient,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::new()?);

    let recorder: Box<dyn RecordingService> = match config.recorder.implementation {
        RecorderImplementation::Local => {
            Box::new(LocalRecorder::new(&config.recorder.device_name)?)
        }
    };

    let transcriber: Box<dyn TranscriptionService> = match config.transcriber.implementation {
        TranscriberImplementation::Local => Box::new(LocalWhisperTranscriber::new(
            &config.transcriber.model_path,
            config.transcriber.use_gpu,
        )?),
    };

    let geocoding_service: Arc<dyn GeocodingService> = match config.geocoding.implementation {
        GeocodingImplementation::Nominatim => Arc::new(NominatimClient::new(
            &config.geocoding.base_url,
            &config.geocoding.user_agent,
        )?),
    };

    let llm_service: Arc<dyn LlmService> = match config.llm.implementation {
        LlmImplementation::Ollama => {
            Arc::new(OllamaClient::new(&config.llm.model, &config.llm.base_url)?)
        }
    };

    let weather_service: Arc<dyn WeatherService> = match config.weather.implementation {
        WeatherImplementation::OpenWeatherMap => Arc::new(OpenWeatherMapClient::new(
            std::env::var("OPENWEATHERMAP_API_KEY")?,
            &config.weather.base_url,
        )?),
    };

    let timer_service = Arc::new(MemoryTimer::new());

    let parsing_service: Box<dyn ParsingService> = match config.parser.implementation {
        ParserImplementation::PatternMatch => Box::new(PatternMatchParser::new()),
        ParserImplementation::Rasa => Box::new(RasaClient::new(&config.parser.rasa_base_url)?),
    };

    let workspace_service = Arc::new(KWinClient);

    let runtime_service = Box::new(LocalRuntime::new(
        geocoding_service,
        llm_service,
        weather_service,
        timer_service,
        workspace_service,
    ));

    let tts_service: Box<dyn TtsService> = match config.tts.implementation {
        TtsImplementation::Piper => {
            Box::new(PiperClient::new(&config.tts.base_url, &config.tts.voice)?)
        }
    };

    let server = WsServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        parsing_service,
        runtime_service,
        tts_service,
        Arc::new(config.response.response_type.clone()),
    )
    .await?;

    server.listen().await?;

    Ok(())
}
