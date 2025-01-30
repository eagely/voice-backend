use crate::{error::Result, model::geocode::GeocodeResponse};
use async_trait::async_trait;

#[async_trait]
pub trait WeatherService: Send + Sync {
    async fn request(&self, geocode: GeocodeResponse) -> Result<String>;
}
