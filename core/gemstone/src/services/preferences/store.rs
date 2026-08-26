use super::error::GemPreferencesError;

#[uniffi::export(with_foreign)]
pub trait GemPreferencesStore: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, GemPreferencesError>;
    fn set(&self, key: String, value: String) -> Result<(), GemPreferencesError>;
    fn remove(&self, key: String) -> Result<(), GemPreferencesError>;
}
