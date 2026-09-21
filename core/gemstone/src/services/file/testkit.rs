use std::sync::Mutex;

use super::GemFileStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryFileStore {
    pub files: Mutex<Vec<String>>,
    pub removed: Mutex<Vec<String>>,
    pub remove_error: Mutex<Option<GemServiceError>>,
}

impl MemoryFileStore {
    pub fn with_file(file_name: &str) -> Self {
        Self {
            files: Mutex::new(vec![file_name.to_string()]),
            ..Default::default()
        }
    }
}

impl GemFileStore for MemoryFileStore {
    fn save_file(&self, _data: Vec<u8>, extension: String) -> Result<String, GemServiceError> {
        let mut files = self.files.lock().unwrap();
        let file_name = format!("file-{}.{extension}", files.len());
        files.push(file_name.clone());
        Ok(file_name)
    }
    fn save_named_file(&self, _data: Vec<u8>, file_name: String) -> Result<String, GemServiceError> {
        self.files.lock().unwrap().push(file_name.clone());
        Ok(file_name)
    }
    fn exists(&self, file_name: String) -> bool {
        self.files.lock().unwrap().contains(&file_name)
    }
    fn path(&self, file_name: String) -> String {
        file_name
    }
    fn remove(&self, file_name: String) -> Result<(), GemServiceError> {
        if let Some(error) = self.remove_error.lock().unwrap().clone() {
            return Err(error);
        }
        self.files.lock().unwrap().retain(|stored| stored != &file_name);
        self.removed.lock().unwrap().push(file_name);
        Ok(())
    }
}

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
