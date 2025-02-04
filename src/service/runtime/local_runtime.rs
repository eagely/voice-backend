use super::runtime_service::RuntimeService;
use crate::error::Result;
use crate::model::action::{Action, IntentKind};
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::weather::WeatherService;
use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::{once, Stream};

pub struct LocalRuntime {
    geocoding_service: Arc<dyn GeocodingService>,
    llm_service: Arc<dyn LlmService>,
    weather_service: Arc<dyn WeatherService>,
}

impl LocalRuntime {
    pub fn new(
        geocoding_service: Arc<dyn GeocodingService>,
        llm_service: Arc<dyn LlmService>,
        weather_service: Arc<dyn WeatherService>,
    ) -> Self {
        Self {
            geocoding_service,
            llm_service,
            weather_service,
        }
    }

    fn string_stream(
        s: impl Into<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Ok(Box::pin(once(Ok(s.into()))))
    }
}

#[async_trait]
impl RuntimeService for LocalRuntime {
    async fn run(
        &self,
        action: Action,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        match action.intent.name {
            IntentKind::LLMQuery => self.llm_service.request(&action.text.to_owned()).await,
            IntentKind::Weather => {
                let weather = match action.entities {
                    Some(entities) => match entities.get("location") {
                        Some(address) => {
                            let geocode = self.geocoding_service.request(&address.name).await?;
                            Self::string_stream(self.weather_service.request(geocode).await?)
                        }
                        None => Self::string_stream("Could you repeat that?"),
                    },
                    None => Self::string_stream("Could you repeat that?"),
                };
                weather
            }
        }
    }
}
