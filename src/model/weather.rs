use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub main: Main,
    pub weather: Vec<Weather>,
}

#[derive(Deserialize, Debug)]
pub struct Main {
    pub temp: f64,
    pub humidity: u8,
}

#[derive(Deserialize, Debug)]
pub struct Weather {
    pub description: String,
}
