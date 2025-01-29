use serde::Deserialize;

#[derive(Deserialize)]
pub struct GeocodeResponse {
    pub name: String,
    pub lat: String,
    pub lon: String,
}
