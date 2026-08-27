use crate::services::error::GemServiceError;

#[uniffi::export(with_foreign)]
pub trait GemFileStore: Send + Sync {
    fn save(&self, data: Vec<u8>, extension: String) -> Result<String, GemServiceError>;
    fn remove(&self, file_name: String) -> Result<(), GemServiceError>;
}
