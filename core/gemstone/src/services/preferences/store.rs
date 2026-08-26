use crate::services::error::GemServiceError;

#[uniffi::export(with_foreign)]
pub trait GemPreferencesStore: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, GemServiceError>;
    fn set(&self, key: String, value: String) -> Result<(), GemServiceError>;
    fn remove(&self, key: String) -> Result<(), GemServiceError>;
}
