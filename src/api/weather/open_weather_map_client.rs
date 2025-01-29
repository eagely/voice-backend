use super::weather_client::WeatherClient;
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
            client: Client::new(),
            api_key: api_key.into(),
            base_url: Url::parse(&base_url.into())?,
        })
    }
}

#[async_trait]
impl WeatherClient for OpenWeatherMapClient {
    async fn request(&self, geocode: GeocodeResponse) -> Result<String> {
        let mut url = self.base_url.clone();
        url.set_path("data/2.5/weather");
        url.query_pairs_mut()
            .append_pair("appid", &self.api_key)
            .append_pair("lat", &geocode.lat.to_string())
            .append_pair("lon", &geocode.lon.to_string())
            .append_pair("units", "metric");

        let weather_response: WeatherResponse =
            serde_json::from_str(&self.client.get(url).send().await?.text().await?)?;

        let description = weather_response
            .weather
            .first()
            .map_or(String::new(), |w| w.description.clone() + " and a");
        let temp = weather_response.main.temp;
        let humidity = weather_response.main.humidity;

        let result = format!(
            "The temperature in {} is {}°C with {} humidity of {}%",
            geocode.name, temp, description, humidity
        );
        Ok(result)
    }
}
