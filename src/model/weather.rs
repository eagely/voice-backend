use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub current: Current,
}

#[derive(Deserialize, Debug)]
pub struct Current {
    pub temp: f64,
    pub humidity: u8,
    pub weather: Vec<Weather>,
}

#[derive(Deserialize, Debug)]
pub struct Weather {
    pub description: String,
}
