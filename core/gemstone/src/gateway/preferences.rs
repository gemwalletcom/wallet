use std::sync::Arc;

use crate::services::preferences::GemPreferencesStore;

pub(crate) struct PreferencesWrapper {
    pub(crate) preferences: Arc<dyn GemPreferencesStore>,
}

impl primitives::Preferences for PreferencesWrapper {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.get(key).map_err(Into::into)
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.set(key, value).map_err(Into::into)
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.remove(key).map_err(Into::into)
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct EmptyPreferences;

#[cfg(test)]
impl GemPreferencesStore for EmptyPreferences {
    fn get(&self, _key: String) -> Result<Option<String>, crate::services::preferences::GemPreferencesError> {
        Ok(None)
    }

    fn set(&self, _key: String, _value: String) -> Result<(), crate::services::preferences::GemPreferencesError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), crate::services::preferences::GemPreferencesError> {
        Ok(())
    }
}
