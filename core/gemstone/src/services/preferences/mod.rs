pub mod error;
pub mod store;

use std::sync::Arc;

pub use error::GemPreferencesError;
pub use store::GemPreferencesStore;

const PRICE_ALERTS_ENABLED: &str = "price_alerts_enabled";

#[derive(uniffi::Object)]
pub struct GemPreferencesService {
    store: Arc<dyn GemPreferencesStore>,
}

#[uniffi::export]
impl GemPreferencesService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemPreferencesStore>) -> Self {
        Self { store }
    }

    pub fn is_price_alerts_enabled(&self) -> Result<bool, GemPreferencesError> {
        self.get_bool(PRICE_ALERTS_ENABLED)
    }

    pub fn set_price_alerts_enabled(&self, enabled: bool) -> Result<(), GemPreferencesError> {
        self.set_bool(PRICE_ALERTS_ENABLED, enabled)
    }
}

impl GemPreferencesService {
    fn get_bool(&self, key: &str) -> Result<bool, GemPreferencesError> {
        Ok(self.store.get(key.to_string())?.as_deref() == Some("true"))
    }

    fn set_bool(&self, key: &str, value: bool) -> Result<(), GemPreferencesError> {
        self.store.set(key.to_string(), value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<HashMap<String, String>>,
    }

    impl GemPreferencesStore for MemoryStore {
        fn get(&self, key: String) -> Result<Option<String>, GemPreferencesError> {
            Ok(self.values.lock().unwrap().get(&key).cloned())
        }

        fn set(&self, key: String, value: String) -> Result<(), GemPreferencesError> {
            self.values.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn remove(&self, key: String) -> Result<(), GemPreferencesError> {
            self.values.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    #[test]
    fn test_price_alerts_enabled_defaults_to_false_and_round_trips() {
        let service = GemPreferencesService::new(Arc::new(MemoryStore::default()));

        assert!(!service.is_price_alerts_enabled().unwrap());

        service.set_price_alerts_enabled(true).unwrap();
        assert!(service.is_price_alerts_enabled().unwrap());

        service.set_price_alerts_enabled(false).unwrap();
        assert!(!service.is_price_alerts_enabled().unwrap());
    }
}
