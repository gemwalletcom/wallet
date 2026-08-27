use std::sync::Arc;

use crate::services::preferences::{GemPreferencesStore, GemSecureStore};

pub(crate) struct PreferencesWrapper {
    pub(crate) preferences: Arc<dyn GemPreferencesStore>,
}

impl primitives::Preferences for PreferencesWrapper {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.preferences.get(key))
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.set(key, value).map_err(Into::into)
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.remove(key).map_err(Into::into)
    }
}

pub(crate) struct SecureStoreWrapper {
    pub(crate) store: Arc<dyn GemSecureStore>,
}

impl primitives::Preferences for SecureStoreWrapper {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.store.get(key).map_err(Into::into)
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store.set(key, value).map_err(Into::into)
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store.remove(key).map_err(Into::into)
    }
}

#[cfg(test)]
use crate::services::error::GemServiceError;

#[cfg(test)]
#[derive(Debug, Default)]
pub struct EmptyPreferences;

#[cfg(test)]
impl GemSecureStore for EmptyPreferences {
    fn get(&self, _key: String) -> Result<Option<String>, GemServiceError> {
        Ok(None)
    }

    fn set(&self, _key: String, _value: String) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), GemServiceError> {
        Ok(())
    }
}

#[cfg(test)]
impl GemPreferencesStore for EmptyPreferences {
    fn get(&self, _key: String) -> Option<String> {
        None
    }

    fn set(&self, _key: String, _value: String) -> Result<(), GemServiceError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), GemServiceError> {
        Ok(())
    }
}
