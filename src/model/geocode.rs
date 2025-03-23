use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GeocodeResponse {
    pub name: String,
    pub lat: String,
    pub lon: String,
}
