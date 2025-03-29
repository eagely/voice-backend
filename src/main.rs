mod config;
mod error;
mod model;
mod server;
mod service;

use crate::error::Result;
use config::{
    enums::{
        GeocodingImplementation, LlmImplementation, ParsingImplementation, RecordingImplementation,
        SynthesisImplementation, TranscriptionImplementation, WeatherImplementation,
    },
    AppConfig,
};
use log::{error, info, warn};
use server::ws::WsServer;
use service::{
    geocoding::{GeocodingService, NominatimClient},
    llm::{deepseek_client::DeepSeekClient, LlmService, OllamaClient},
    parsing::{ParsingService, PatternMatchParser, RasaClient},
    recording::{remote_recorder::RemoteRecorder, LocalRecorder, RecordingService},
    runtime::LocalRuntime,
    synthesis::{ElevenLabsClient, PiperClient, SynthesizerService},
    timer::memory_timer::MemoryTimer,
    transcription::{DeepgramClient, LocalWhisperClient, TranscriptionService},
    volume::PactlClient,
    weather::{OpenWeatherMapClient, WeatherService},
    workspace::KWinClient,
};
use std::{env::var, process, sync::Arc};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    loop {
        match run_server().await {
            Ok(_) => {
                error!("Server stopped unexpectedly");
            }
            Err(e) => {
                error!("Server error: {}", e);
            }
        }

        warn!("Attempting to restart server in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_server() -> Result<()> {
    info!("Starting voice assistant server...");

    let config = match AppConfig::new() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    let recorder = initialize_recorder(&config).await?;
    let transcriber = initialize_transcriber(&config).await?;
    let geocoding_service = initialize_geocoding_service(&config).await?;
    let llm_service = initialize_llm_service(&config).await?;
    let weather_service = initialize_weather_service(&config).await?;
    let timer_service = Arc::new(MemoryTimer::new());
    let parsing_service = initialize_parsing_service(&config).await?;
    let volume_service = Arc::new(PactlClient);
    let workspace_service = Arc::new(KWinClient);
    let synthesis_service = initialize_synthesis_service(&config)?;

    let runtime_service = Box::new(LocalRuntime::new(
        geocoding_service,
        llm_service,
        weather_service,
        timer_service,
        volume_service,
        workspace_service,
    ));

    info!("Initializing WebSocket server...");
    let server = WsServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        parsing_service,
        runtime_service,
        synthesis_service,
        config.response.response_kind.clone(),
    )
    .await?;

    info!("Server started successfully");
    server.listen().await
}

async fn initialize_recorder(config: &Arc<AppConfig>) -> Result<Box<dyn RecordingService>> {
    info!("Initializing recording service...");
    match config.recording.implementation {
        RecordingImplementation::Local => Ok(Box::new(LocalRecorder::new(
            &config.recording.device_name,
            var("PICOVOICE_ACCESS_KEY")?,
            &config.recording.wake_word,
            config.recording.wake_word_enabled,
            config.recording.porcupine_sensitivity,
        )?)),
        RecordingImplementation::Remote => {
            match RemoteRecorder::new(&config.recording.remote_url).await {
                Ok(recorder) => Ok(Box::new(recorder)),
                Err(e) => {
                    error!("Failed to initialize remote recorder: {}", e);
                    warn!("Falling back to local recorder");
                    Ok(Box::new(LocalRecorder::new(
                        &config.recording.device_name,
                        var("PICOVOICE_ACCESS_KEY")?,
                        &config.recording.wake_word,
                        config.recording.wake_word_enabled,
                        config.recording.porcupine_sensitivity,
                    )?))
                }
            }
        }
    }
}

async fn initialize_transcriber(config: &Arc<AppConfig>) -> Result<Box<dyn TranscriptionService>> {
    info!("Initializing transcription service...");
    match config.transcription.implementation {
        TranscriptionImplementation::Deepgram => {
            match DeepgramClient::new(&config.transcription.deepgram_base_url) {
                Ok(client) => Ok(Box::new(client)),
                Err(e) => {
                    error!("Failed to initialize Deepgram client: {}", e);
                    warn!("Falling back to local transcription");
                    Ok(Box::new(LocalWhisperClient::new(
                        &config.transcription.local_model_path,
                        config.transcription.local_use_gpu,
                    )?))
                }
            }
        }
        TranscriptionImplementation::Local => Ok(Box::new(LocalWhisperClient::new(
            &config.transcription.local_model_path,
            config.transcription.local_use_gpu,
        )?)),
    }
}

async fn initialize_geocoding_service(
    config: &Arc<AppConfig>,
) -> Result<Arc<dyn GeocodingService>> {
    info!("Initializing geocoding service...");
    match config.geocoding.implementation {
        GeocodingImplementation::Nominatim => Ok(Arc::new(NominatimClient::new(
            &config.geocoding.base_url,
            &config.geocoding.user_agent,
        )?)),
    }
}

async fn initialize_llm_service(config: &Arc<AppConfig>) -> Result<Arc<dyn LlmService>> {
    info!("Initializing LLM service...");
    match config.llm.implementation {
        LlmImplementation::DeepSeek => {
            match DeepSeekClient::new(&config.llm.deepseek_model, &config.llm.deepseek_base_url) {
                Ok(client) => Ok(Arc::new(client)),
                Err(e) => {
                    error!("Failed to initialize DeepSeek client: {}", e);
                    warn!("Falling back to Ollama");
                    Ok(Arc::new(OllamaClient::new(
                        &config.llm.ollama_model,
                        &config.llm.ollama_base_url,
                    )?))
                }
            }
        }
        LlmImplementation::Ollama => Ok(Arc::new(OllamaClient::new(
            &config.llm.ollama_model,
            &config.llm.ollama_base_url,
        )?)),
    }
}

async fn initialize_weather_service(config: &Arc<AppConfig>) -> Result<Arc<dyn WeatherService>> {
    info!("Initializing weather service...");
    match config.weather.implementation {
        WeatherImplementation::OpenWeatherMap => {
            let api_key = var("OPENWEATHERMAP_API_KEY").map_err(|e| {
                error!("OpenWeatherMap API key not found: {}", e);
                e
            })?;
            Ok(Arc::new(OpenWeatherMapClient::new(
                api_key,
                &config.weather.base_url,
            )?))
        }
    }
}

async fn initialize_parsing_service(config: &Arc<AppConfig>) -> Result<Box<dyn ParsingService>> {
    info!("Initializing parsing service...");
    match config.parsing.implementation {
        ParsingImplementation::PatternMatch => Ok(Box::new(PatternMatchParser::new())),
        ParsingImplementation::Rasa => match RasaClient::new(&config.parsing.rasa_base_url) {
            Ok(client) => Ok(Box::new(client)),
            Err(e) => {
                error!("Failed to initialize Rasa client: {}", e);
                warn!("Falling back to pattern matching parser");
                Ok(Box::new(PatternMatchParser::new()))
            }
        },
    }
}

fn initialize_synthesis_service(config: &Arc<AppConfig>) -> Result<Box<dyn SynthesizerService>> {
    info!("Initializing synthesis service...");
    match config.synthesis.implementation {
        SynthesisImplementation::Elevenlabs => {
            match ElevenLabsClient::new(
                &config.synthesis.elevenlabs_base_url,
                &config.synthesis.elevenlabs_model_id,
                &config.synthesis.elevenlabs_voice_id,
            ) {
                Ok(client) => Ok(Box::new(client)),
                Err(e) => {
                    error!("Failed to initialize ElevenLabs client: {}", e);
                    warn!("Falling back to Piper");
                    Ok(Box::new(PiperClient::new(
                        &config.synthesis.piper_base_url,
                        &config.synthesis.piper_voice,
                    )?))
                }
            }
        }
        SynthesisImplementation::Piper => Ok(Box::new(PiperClient::new(
            &config.synthesis.piper_base_url,
            &config.synthesis.piper_voice,
        )?)),
    }
}
