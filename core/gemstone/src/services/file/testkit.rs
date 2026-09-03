use super::GemFileStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct NoopFileStore;

impl GemFileStore for NoopFileStore {
    fn save_file(&self, _data: Vec<u8>, _extension: String) -> Result<String, GemServiceError> {
        Ok(String::new())
    }
    fn save_named_file(&self, _data: Vec<u8>, _file_name: String) -> Result<String, GemServiceError> {
        Ok(String::new())
    }
    fn exists(&self, _file_name: String) -> bool {
        false
    }
    fn path(&self, file_name: String) -> String {
        file_name
    }
    fn remove(&self, _file_name: String) -> Result<(), GemServiceError> {
        Ok(())
    }
}
