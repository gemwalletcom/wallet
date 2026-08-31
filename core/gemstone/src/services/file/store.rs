use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
pub trait GemFileStore: Send + Sync {
    fn save_file(&self, data: Vec<u8>, extension: String) -> Result<String, GemServiceError>;
    fn save_named_file(&self, data: Vec<u8>, file_name: String) -> Result<String, GemServiceError>;
    fn exists(&self, file_name: String) -> bool;
    fn path(&self, file_name: String) -> String;
    fn remove(&self, file_name: String) -> Result<(), GemServiceError>;
}
