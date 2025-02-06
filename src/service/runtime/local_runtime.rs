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
        if let Some(confidence) = action.intent.confidence {
            if action.intent.name != IntentKind::LlmQuery && confidence < 0.9 {
                return Self::string_stream(
                    "I'm not sure if I understood you correctly. Could you say that again?",
                );
            }
        }
        match action.intent.name {
            IntentKind::LlmQuery => self.llm_service.request(&action.text.to_string()).await,
            IntentKind::WeatherQuery => {
                let weather = match action.entities.iter()
                    .find(|entity| entity.entity == "GPE")
                    .or_else(|| action.entities.iter().find(|entity| entity.entity == "location")) {
                    Some(location) => {
                        if let Some(confidence) = location.confidence {
                            if confidence < 0.9 {
                                return Self::string_stream("I'm not sure which location you are referring to. Could you say that again?");
                            }
                        }
                        let geocode = self.geocoding_service.request(&location.value).await?;
                        Self::string_stream(self.weather_service.request(geocode).await?)
                    }
                    None => Self::string_stream("I couldn't figure out which location you were referring to, Could you say that again?"),
                };
                weather
            }
        }
    }
}
