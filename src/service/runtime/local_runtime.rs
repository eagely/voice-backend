use super::runtime_service::RuntimeService;
use crate::error::Result;
use crate::model::action::{Action, EntityValue, IntentKind};
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::timer::timer_service::TimerService;
use crate::service::weather::WeatherService;
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use std::sync::Arc;
use std::time::Duration;

pub struct LocalRuntime {
    geocoding_service: Arc<dyn GeocodingService>,
    llm_service: Arc<dyn LlmService>,
    weather_service: Arc<dyn WeatherService>,
    timer_service: Arc<dyn TimerService>,
}

impl LocalRuntime {
    pub fn new(
        geocoding_service: Arc<dyn GeocodingService>,
        llm_service: Arc<dyn LlmService>,
        weather_service: Arc<dyn WeatherService>,
        timer_service: Arc<dyn TimerService>,
    ) -> Self {
        Self {
            geocoding_service,
            llm_service,
            weather_service,
            timer_service,
        }
    }

    fn string_stream(
        s: impl Into<String> + Send + 'static,
    ) -> Result<BoxStream<'static, Result<String>>> {
        Ok(stream::once(async { Ok(s.into()) }).boxed())
    }
}

#[async_trait]
impl RuntimeService for LocalRuntime {
    async fn run(&self, action: Action) -> Result<BoxStream<'static, Result<String>>> {
        if let Some(confidence) = action.intent.confidence {
            if action.intent.name != IntentKind::LlmQuery && confidence < 0.9 {
                return Self::string_stream(
                    "I'm not sure if I understood you correctly. Could you say that again?",
                );
            }
        }

        match action.intent.name {
            IntentKind::LlmQuery => {
                let response = self.llm_service.request(&action.text).await;
                response
            }
            IntentKind::SetTimer => {
                let duration = action
                    .entities
                    .iter()
                    .find(|e| e.entity == "duration")
                    .and_then(|e| match &e.value {
                        EntityValue::Duration(d) => Some(Duration::from_secs(d.value)),
                        _ => None,
                    });

                let response = match duration {
                    Some(duration) => self.timer_service.set(duration, action.text).await?,
                    None => "Please specify a clear duration for the timer.".to_string(),
                };

                Self::string_stream(response)
            }
            IntentKind::WeatherQuery => {
                let location_entity = action
                    .entities
                    .iter()
                    .find(|entity| entity.entity == "GPE")
                    .or_else(|| {
                        action
                            .entities
                            .iter()
                            .find(|entity| entity.entity == "location")
                    });

                let response = match location_entity {
                    Some(entity) => {
                        if let Some(confidence) = entity.confidence {
                            if confidence < 0.9 {
                                return Self::string_stream("I'm not sure which location you are referring to. Could you say that again?");
                            }
                        }

                        match &entity.value {
                            EntityValue::String(location) => {
                                let geocode = self.geocoding_service.request(location).await?;
                                let weather_response = self.weather_service.request(geocode).await?;
                                Self::string_stream(weather_response)
                            },
                            _ => Self::string_stream("Invalid location format received.")
                        }
                    }
                    None => Self::string_stream("I couldn't figure out which location you were referring to. Could you say that again?"),
                };
                response
            }
            IntentKind::Other(intent_kind) => {
                let response = format!("The intent {} is not implemented.", intent_kind);
                Self::string_stream(response)
            }
        }
    }
}
