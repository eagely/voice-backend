use super::runtime_service::RuntimeService;
use crate::error::Result;
use crate::model::action::{Action, EntityValue, IntentKind};
use crate::service::geocoding::GeocodingService;
use crate::service::llm::LlmService;
use crate::service::timer::timer_service::TimerService;
use crate::service::volume::VolumeService;
use crate::service::weather::WeatherService;
use crate::service::workspace::WorkspaceService;
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use log::info;
use std::sync::Arc;
use std::time::Duration;

pub struct LocalRuntime {
    geocoding_service: Arc<dyn GeocodingService>,
    llm_service: Arc<dyn LlmService>,
    weather_service: Arc<dyn WeatherService>,
    timer_service: Arc<dyn TimerService>,
    volume_service: Arc<dyn VolumeService>,
    workspace_service: Arc<dyn WorkspaceService>,
}

impl LocalRuntime {
    pub fn new(
        geocoding_service: Arc<dyn GeocodingService>,
        llm_service: Arc<dyn LlmService>,
        weather_service: Arc<dyn WeatherService>,
        timer_service: Arc<dyn TimerService>,
        volume_service: Arc<dyn VolumeService>,
        workspace_service: Arc<dyn WorkspaceService>,
    ) -> Self {
        Self {
            geocoding_service,
            llm_service,
            weather_service,
            timer_service,
            volume_service,
            workspace_service,
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
            IntentKind::CloseWindow => {
                self.workspace_service.close_window().await?;
                Self::string_stream("Window closed.")
            }
            IntentKind::LlmQuery => {
                let response = self.llm_service.request(&action.text).await;
                response
            }
            IntentKind::DecreaseVolume => {
                let value = action.entities.iter().find(|e| e.entity == "NUMBER");

                if let Some(value) = value {
                    if let EntityValue::Index(index) = value.value {
                        self.volume_service.decrease(index as u8).await?;
                        Self::string_stream("Volume decreased.")
                    } else {
                        Self::string_stream("Invalid volume value provided.")
                    }
                } else {
                    Self::string_stream("No value specified for volume decrease.")
                }
            }
            IntentKind::IncreaseVolume => {
                let value = action.entities.iter().find(|e| e.entity == "NUMBER");

                if let Some(value) = value {
                    if let EntityValue::Index(index) = value.value {
                        self.volume_service.increase(index as u8).await?;
                        Self::string_stream("Volume increased.")
                    } else {
                        Self::string_stream("Invalid volume value provided.")
                    }
                } else {
                    Self::string_stream("No value specified for volume increase.")
                }
            }
            IntentKind::SetVolume => {
                let value = action.entities.iter().find(|e| e.entity == "NUMBER");

                if let Some(value) = value {
                    if let EntityValue::Index(index) = value.value {
                        self.volume_service.set(index as u8).await?;
                        Self::string_stream("Volume set.")
                    } else {
                        Self::string_stream("Invalid volume value provided.")
                    }
                } else {
                    Self::string_stream("No value specified for volume setting.")
                }
            }
            IntentKind::MinimizeWindow => {
                self.workspace_service.minimize_window().await?;
                Self::string_stream("Window minimized.")
            }
            IntentKind::MaximizeWindow => {
                self.workspace_service.maximize_window().await?;
                Self::string_stream("Window maximized.")
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
            IntentKind::ShowDesktop => {
                self.workspace_service.show_desktop().await?;
                Self::string_stream("Desktop shown.")
            }
            IntentKind::SwitchWorkspace => {
                let workspace_entity = action
                    .entities
                    .iter()
                    .find(|entity| entity.entity == "workspace");

                let response = match workspace_entity {
                    Some(entity) => {
                        if let Some(confidence) = entity.confidence {
                            if confidence < 0.9 {
                                return Self::string_stream("I'm not sure which workspace you are referring to. Could you say that again?");
                            }
                        }
                        if let EntityValue::Index(index) = entity.value {
                            self.workspace_service.switch_workspace(index).await?;
                            format!("Switched to workspace {}.", index)
                        } else {
                            "Please specify a workspace to switch to.".to_string()
                        }
                    }
                    None => "Please specify a clear workspace to switch to.".to_string(),
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
                                info!("Geocode received: {:?}", &geocode);
                                let weather_response = self.weather_service.request(geocode).await?;
                                info!("Weather response received: {:?}", &weather_response);
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
