use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
pub trait GemPreferencesStore: Send + Sync {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String) -> Result<(), GemServiceError>;
    fn remove(&self, key: String) -> Result<(), GemServiceError>;
}

#[uniffi::export(rust, foreign)]
pub trait GemSecureStore: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, GemServiceError>;
    fn set(&self, key: String, value: String) -> Result<(), GemServiceError>;
    fn remove(&self, key: String) -> Result<(), GemServiceError>;
}
