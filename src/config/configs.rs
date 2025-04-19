use super::enums::{
    GeocodingImplementation, LlmImplementation, ParsingImplementation, RecordingImplementation,
    ResponseKind, SynthesisImplementation, TranscriptionImplementation, WeatherImplementation,
};
use crate::error::Result;
use config::{Config, File};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs::{read_to_string, write};
use toml::{to_string, Value};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub geocoding: GeocodingConfig,
    pub llm: LlmConfig,
    pub parsing: ParsingConfig,
    pub recording: RecordingConfig,
    pub response: ResponseConfig,
    pub server: ServerConfig,
    pub transcription: TranscriptionConfig,
    pub synthesis: SynthesisConfig,
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
    pub ollama_base_url: String,
    pub deepseek_base_url: String,
    pub deepseek_model: String,
    pub ollama_model: String,
    pub implementation: LlmImplementation,
}

#[derive(Debug, Deserialize)]
pub struct ParsingConfig {
    pub rasa_base_url: String,
    pub implementation: ParsingImplementation,
}

#[derive(Debug, Deserialize)]
pub struct RecordingConfig {
    pub device_name: String,
    pub implementation: RecordingImplementation,
    pub remote_url: String,
    pub porcupine_sensitivity: f32,
    pub wake_word: String,
    pub wake_word_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResponseConfig {
    pub response_kind: ResponseKind,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptionConfig {
    pub deepgram_base_url: String,
    pub local_model_path: String,
    pub local_use_gpu: bool,
    pub implementation: TranscriptionImplementation,
}

#[derive(Debug, Deserialize)]
pub struct SynthesisConfig {
    pub elevenlabs_base_url: String,
    pub elevenlabs_model_id: String,
    pub elevenlabs_voice_id: String,
    pub piper_base_url: String,
    pub piper_voice: String,
    pub implementation: SynthesisImplementation,
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

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }

    pub async fn get_all_config_entries() -> Result<Vec<String>> {
        let mut builder = Config::builder().add_source(File::from_str(
            include_str!("default.toml"),
            config::FileFormat::Toml,
        ));

        if let Some(path) = Self::get_config_file() {
            if !path.exists() {
                Self::create_default_config_file(&path)?;
            }
            builder = builder.add_source(File::from(path).required(false));
        }

        let cfg = builder.build()?;
        let toml_val: Value = cfg.try_deserialize()?;
        let top = toml_val.as_table().ok_or_else(|| {
            crate::error::Error::ConfigError(config::ConfigError::Message(
                "invalid config structure".into(),
            ))
        })?;

        let mut entries = Vec::new();
        for (table_name, table_val) in top {
            if let Value::Table(inner) = table_val {
                for (key, val) in inner {
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        Value::Integer(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Boolean(b) => b.to_string(),
                        other => other.to_string(),
                    };
                    entries.push(format!("{}.{}={}", table_name, key, val_str));
                }
            }
        }

        Ok(entries)
    }

    pub async fn write_config(table: &str, key: &str, value: &str) -> Result<()> {
        if let Some(config_path) = Self::get_config_file() {
            let config_content = read_to_string(&config_path).await?;
            let mut config_value: Value = config_content.parse()?;

            let table_value = config_value.get_mut(table).ok_or_else(|| {
                crate::error::Error::ConfigError(config::ConfigError::Message(format!(
                    "Table not found: {}",
                    table
                )))
            })?;

            table_value[key] = Value::String(value.to_string());

            let new_config_content = to_string(&config_value)?;
            write(config_path, new_config_content).await?;

            Ok(())
        } else {
            Err(crate::error::Error::ConfigError(
                config::ConfigError::Message("Configuration file path not found".to_string()),
            ))
        }
    }
}
