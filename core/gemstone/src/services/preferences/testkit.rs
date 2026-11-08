use std::collections::HashMap;
use std::sync::Mutex;

use super::{GemPreferencesStore, GemSecureStore};
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryPreferencesStore {
    pub values: Mutex<HashMap<String, String>>,
}

impl GemPreferencesStore for MemoryPreferencesStore {
    fn get(&self, key: String) -> Option<String> {
        self.values.lock().unwrap().get(&key).cloned()
    }

    fn set(&self, key: String, value: String) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().insert(key, value);
        Ok(())
    }

    fn remove(&self, key: String) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().remove(&key);
        Ok(())
    }

    fn clear(&self) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().clear();
        Ok(())
    }
}

#[derive(Default)]
pub struct MemorySecureStore {
    pub values: Mutex<HashMap<String, String>>,
}

impl GemSecureStore for MemorySecureStore {
    fn get(&self, key: String) -> Result<Option<String>, GemServiceError> {
        Ok(self.values.lock().unwrap().get(&key).cloned())
    }

    fn set(&self, key: String, value: String) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().insert(key, value);
        Ok(())
    }

    fn remove(&self, key: String) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().remove(&key);
        Ok(())
    }
}
