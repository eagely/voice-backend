use crate::error::Result;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;
use toml::{to_string, Value};

use super::enums::{GeocodingImplementation, LlmImplementation, ParserImplementation, RecorderImplementation, ResponseKind, TranscriberImplementation, TtsImplementation, WeatherImplementation};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub geocoding: GeocodingConfig,
    pub llm: LlmConfig,
    pub parser: ParserConfig,
    pub recorder: RecorderConfig,
    pub response: ResponseConfig,
    pub server: ServerConfig,
    pub transcriber: TranscriberConfig,
    pub tts: TtsConfig,
    pub weather: WeatherConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeocodingConfig {
    pub base_url: String,
    pub user_agent: String,
    pub implementation: GeocodingImplementation,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub implementation: LlmImplementation,
}

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub rasa_base_url: String,
    pub implementation: ParserImplementation,
}

#[derive(Debug, Deserialize)]
pub struct RecorderConfig {
    pub device_name: String,
    pub implementation: RecorderImplementation,
}

#[derive(Debug, Deserialize)]
pub struct ResponseConfig {
    pub response_type: ResponseKind,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct TranscriberConfig {
    pub model_path: String,
    pub use_gpu: bool,
    pub implementation: TranscriberImplementation,
}

#[derive(Debug, Deserialize)]
pub struct TtsConfig {
    pub base_url: String,
    pub voice: String,
    pub implementation: TtsImplementation,
}

#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub base_url: String,
    pub implementation: WeatherImplementation,
}

impl AppConfig {
    fn get_config_file() -> Option<PathBuf> {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(|xdg_config_home| PathBuf::from(xdg_config_home).join("voice/config.toml"))
    }

    fn create_default_config_file(config_path: &PathBuf) -> Result<()> {
        let default_config = include_str!("default.toml");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(config_path, default_config)?;
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut builder = Config::builder();

        builder = builder.add_source(config::File::from_str(
            include_str!("default.toml"),
            config::FileFormat::Toml,
        ));

        if let Some(config_path) = Self::get_config_file() {
            if !config_path.exists() {
                Self::create_default_config_file(&config_path)?;
            }
            builder = builder.add_source(File::from(config_path).required(false));
        }

        builder = builder.add_source(Environment::with_prefix("APP").separator("_"));

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }

    pub async fn write_config(table: &str, key: &str, value: &str) -> Result<()> {
        if let Some(config_path) = Self::get_config_file() {
            let config_content = fs::read_to_string(&config_path).await?;
            let mut config_value: Value = config_content.parse()?;

            let table_value = config_value.get_mut(table).ok_or_else(|| {
                crate::error::Error::Config(config::ConfigError::Message(format!(
                    "Table not found: {}",
                    table
                )))
            })?;

            table_value[key] = Value::String(value.to_string());

            let new_config_content = to_string(&config_value)?;
            fs::write(config_path, new_config_content).await?;

            Ok(())
        } else {
            Err(crate::error::Error::Config(config::ConfigError::Message(
                "Configuration file path not found".to_string(),
            )))
        }
    }
}
