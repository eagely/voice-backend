use async_trait::async_trait;
use crate::error::Result;
use crate::model::geocode::GeocodeResponse;

#[async_trait]
pub trait GeocodingService: Send + Sync {
    async fn request(&self, address: &str) -> Result<GeocodeResponse>;
}
