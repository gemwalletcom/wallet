pub mod error;

use std::sync::Arc;

use primitives::ConfigResponse;

use crate::api::{GemApiClient, GemApiError};
use crate::services::preferences::GemPreferencesService;

pub use error::GemConfigError;

#[derive(uniffi::Object)]
pub struct GemConfigService {
    api: Arc<GemApiClient>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemConfigService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { api, preferences }
    }

    pub async fn get_config(&self) -> Result<ConfigResponse, GemConfigError> {
        match self.preferences.get_config()? {
            Some(config) => Ok(config),
            None => self.update_config().await,
        }
    }

    pub async fn update_config(&self) -> Result<ConfigResponse, GemConfigError> {
        let config = self.api.client.get_config().await.map_err(GemApiError::from)?;
        self.preferences.set_config(&config)?;
        Ok(config)
    }
}
