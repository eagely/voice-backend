use crate::error::Result;
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub geocoding: GeocodingConfig,
    pub llm: LlmConfig,
    pub rasa: RasaConfig,
    pub recorder: RecorderConfig,
    pub response: ResponseConfig,
    pub server: ServerConfig,
    pub transcriber: TranscriberConfig,
    pub tts: TtsConfig,
    pub user: UserConfig,
    pub weather: WeatherConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeocodingConfig {
    pub base_url: String,
    pub user_agent: String,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct RasaConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RecorderConfig {
    pub device_name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Text,
    Audio,
}

#[derive(Debug, Deserialize)]
pub struct ResponseConfig {
    pub response_type: ResponseType,
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
}

#[derive(Debug, Deserialize)]
pub struct TtsConfig {
    pub base_url: String,
    pub voice: String,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub default_location: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub base_url: String,
}

impl AppConfig {
    fn get_config_files() -> Vec<PathBuf> {
        let mut configs = Vec::new();

        #[cfg(unix)]
        {
            configs.push(PathBuf::from("/etc/voice/config.toml"));
        }

        if let Some(proj_dirs) = ProjectDirs::from("dev", "eagely", "voice") {
            configs.push(proj_dirs.config_dir().join("config.toml"));
        }

        configs.push(PathBuf::from("config/config.toml"));

        configs
    }

    pub fn new() -> Result<Self> {
        let mut builder = Config::builder();

        builder = builder.add_source(config::File::from_str(
            include_str!("default.toml"),
            config::FileFormat::Toml,
        ));

        for config_path in Self::get_config_files() {
            builder = builder.add_source(File::from(config_path).required(false));
        }

        builder = builder.add_source(Environment::with_prefix("APP").separator("_"));

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }
}
