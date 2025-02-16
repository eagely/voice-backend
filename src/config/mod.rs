use crate::error::Result;
use config::{Config, Environment, File};
use directories::ProjectDirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct RecorderConfig {
    pub device_name: String,
}

#[derive(Debug, Deserialize)]
pub struct TranscriberConfig {
    pub model_path: String,
    pub use_gpu: bool,
}

#[derive(Debug, Deserialize)]
pub struct GeocodingConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RasaConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub recorder: RecorderConfig,
    pub transcriber: TranscriberConfig,
    pub geocoding: GeocodingConfig,
    pub llm: LlmConfig,
    pub weather: WeatherConfig,
    pub rasa: RasaConfig,
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
