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
use server::ws::WsServer;
use service::{
    geocoding::{GeocodingService, NominatimClient},
    llm::{deepseek_client::DeepSeekClient, LlmService, OllamaClient},
    parsing::{ParsingService, PatternMatchParser, RasaClient},
    recording::{remote_recorder::RemoteRecorder, LocalRecorder, RecordingService},
    runtime::LocalRuntime,
    synthesis::{ElevenlabsClient, PiperClient, SynthesizerService},
    timer::memory_timer::MemoryTimer,
    transcription::{DeepgramClient, LocalWhisperClient, TranscriptionService},
    weather::{OpenWeatherMapClient, WeatherService},
    workspace::KWinClient,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(AppConfig::new()?);

    let recorder: Box<dyn RecordingService> = match config.recorder.implementation {
        RecordingImplementation::Local => {
            Box::new(LocalRecorder::new(&config.recorder.device_name)?)
        }
        RecordingImplementation::Remote => {
            Box::new(RemoteRecorder::new(&config.recorder.remote_url).await?)
        }
    };

    let transcriber: Box<dyn TranscriptionService> = match config.transcriber.implementation {
        TranscriptionImplementation::Deepgram => {
            Box::new(DeepgramClient::new(&config.transcriber.deepgram_base_url)?)
        }
        TranscriptionImplementation::Local => Box::new(LocalWhisperClient::new(
            &config.transcriber.local_model_path,
            config.transcriber.local_use_gpu,
        )?),
    };

    let geocoding_service: Arc<dyn GeocodingService> = match config.geocoding.implementation {
        GeocodingImplementation::Nominatim => Arc::new(NominatimClient::new(
            &config.geocoding.base_url,
            &config.geocoding.user_agent,
        )?),
    };

    let llm_service: Arc<dyn LlmService> = match config.llm.implementation {
        LlmImplementation::DeepSeek => Arc::new(DeepSeekClient::new(
            &config.llm.deepseek_model,
            &config.llm.deepseek_base_url,
        )?),
        LlmImplementation::Ollama => Arc::new(OllamaClient::new(
            &config.llm.ollama_model,
            &config.llm.ollama_base_url,
        )?),
    };

    let weather_service: Arc<dyn WeatherService> = match config.weather.implementation {
        WeatherImplementation::OpenWeatherMap => Arc::new(OpenWeatherMapClient::new(
            std::env::var("OPENWEATHERMAP_API_KEY")?,
            &config.weather.base_url,
        )?),
    };

    let timer_service = Arc::new(MemoryTimer::new());

    let parsing_service: Box<dyn ParsingService> = match config.parser.implementation {
        ParsingImplementation::PatternMatch => Box::new(PatternMatchParser::new()),
        ParsingImplementation::Rasa => Box::new(RasaClient::new(&config.parser.rasa_base_url)?),
    };

    let workspace_service = Arc::new(KWinClient);

    let runtime_service = Box::new(LocalRuntime::new(
        geocoding_service,
        llm_service,
        weather_service,
        timer_service,
        workspace_service,
    ));

    let synthesis_service: Box<dyn SynthesizerService> = match config.synthesizer.implementation {
        SynthesisImplementation::Elevenlabs => Box::new(ElevenlabsClient::new(
            &config.synthesizer.elevenlabs_base_url,
            &config.synthesizer.elevenlabs_model_id,
            &config.synthesizer.elevenlabs_voice_id,
        )?),
        SynthesisImplementation::Piper => Box::new(PiperClient::new(
            &config.synthesizer.piper_base_url,
            &config.synthesizer.piper_voice,
        )?),
    };

    let server = WsServer::new(
        &format!("{}:{}", config.server.host, config.server.port),
        recorder,
        transcriber,
        parsing_service,
        runtime_service,
        synthesis_service,
        config.response.response_type.clone(),
    )
    .await?;

    server.listen().await?;

    Ok(())
}
