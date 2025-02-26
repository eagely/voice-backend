use super::weather_service::WeatherService;
use crate::{
    error::Result,
    model::{geocode::GeocodeResponse, weather::WeatherResponse},
};
use async_trait::async_trait;
use reqwest::{Client, Url};

pub struct OpenWeatherMapClient {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl OpenWeatherMapClient {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            api_key: api_key.into(),
            base_url: Url::parse(&base_url.into())?,
        })
    }
}

#[async_trait]
impl WeatherService for OpenWeatherMapClient {
    async fn request(&self, geocode: GeocodeResponse) -> Result<String> {
        let mut url = self.base_url.clone();
        url.set_path("data/3.0/onecall");
        url.query_pairs_mut()
            .append_pair("appid", &self.api_key)
            .append_pair("lat", &geocode.lat.to_string())
            .append_pair("lon", &geocode.lon.to_string())
            .append_pair("units", "metric");

        let weather_response: WeatherResponse =
            serde_json::from_str(&self.client.get(url).send().await?.text().await?)?;

        let description = weather_response
            .current
            .weather
            .first()
            .map_or(String::new(), |w| w.description.clone() + " and a");
        let temp = weather_response.current.temp;
        let humidity = weather_response.current.humidity;

        let result = format!(
            "The temperature in {} is {}°C with {} humidity of {}%",
            geocode.name, temp, description, humidity
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::model::geocode::GeocodeResponse;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_open_weather_map_client() -> Result<()> {
        let config = Arc::new(AppConfig::new()?);

        let client = OpenWeatherMapClient::new(
            std::env::var("OPENWEATHERMAP_API_KEY")?,
            &config.weather.base_url,
        )?;

        let geocode = GeocodeResponse {
            name: "Vienna".to_string(),
            lat: "48.2082".to_string(),
            lon: "16.3738".to_string(),
        };

        let response = client.request(geocode).await?;

        assert!(response.contains("Vienna"));
        assert!(response.contains("temperature"));
        assert!(response.contains("humidity"));

        Ok(())
    }
}
