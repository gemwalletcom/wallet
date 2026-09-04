use std::sync::{Arc, Mutex};

use gem_api::DeviceKey;
use gem_client::ClientError;

use crate::device::{GemDeviceKeyPair, device_public_key, generate_device_key_pair};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemSecureStore;

const DEVICE_PRIVATE_KEY: &str = "device_private_key";
const DEVICE_PUBLIC_KEY: &str = "device_public_key";

#[derive(uniffi::Object)]
pub struct GemDeviceKeyService {
    store: Arc<dyn GemSecureStore>,
    cached: Mutex<Option<GemDeviceKeyPair>>,
}

#[uniffi::export]
impl GemDeviceKeyService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemSecureStore>) -> Self {
        Self { store, cached: Mutex::new(None) }
    }

    pub fn device_id(&self) -> Result<String, GemServiceError> {
        Ok(hex::encode(self.key_pair()?.public_key))
    }

    pub fn key_pair(&self) -> Result<GemDeviceKeyPair, GemServiceError> {
        let mut cached = match self.cached.lock() {
            Ok(cached) => cached,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(key_pair) = cached.as_ref() {
            return Ok(key_pair.clone());
        }
        let key_pair = match self.stored_private_key()? {
            Some(private_key) => self.key_pair_from(private_key)?,
            None => self.create()?,
        };
        *cached = Some(key_pair.clone());
        Ok(key_pair)
    }
}

impl DeviceKey for GemDeviceKeyService {
    fn private_key(&self) -> Result<Vec<u8>, ClientError> {
        Ok(self.key_pair().map_err(|error| ClientError::Network(error.to_string()))?.private_key)
    }
}

impl std::fmt::Debug for GemDeviceKeyService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GemDeviceKeyService")
    }
}

impl GemDeviceKeyService {
    fn stored_private_key(&self) -> Result<Option<Vec<u8>>, GemServiceError> {
        let Some(value) = self.store.get(DEVICE_PRIVATE_KEY.to_string())? else {
            return Ok(None);
        };
        let private_key = hex::decode(value).map_err(|error| GemServiceError::Core {
            msg: format!("stored device key is not hex: {error}"),
        })?;
        Ok(Some(private_key))
    }

    fn key_pair_from(&self, private_key: Vec<u8>) -> Result<GemDeviceKeyPair, GemServiceError> {
        let public_key = device_public_key(private_key.clone()).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        if self.store.get(DEVICE_PUBLIC_KEY.to_string())?.is_none() {
            self.store.set(DEVICE_PUBLIC_KEY.to_string(), hex::encode(&public_key))?;
        }
        Ok(GemDeviceKeyPair { private_key, public_key })
    }

    fn create(&self) -> Result<GemDeviceKeyPair, GemServiceError> {
        let key_pair = generate_device_key_pair();
        self.store.set(DEVICE_PRIVATE_KEY.to_string(), hex::encode(&key_pair.private_key))?;
        self.store.set(DEVICE_PUBLIC_KEY.to_string(), hex::encode(&key_pair.public_key))?;
        Ok(key_pair)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<HashMap<String, String>>,
        fails: bool,
        missing_private_key_read_delay: Option<Duration>,
        writes: AtomicUsize,
    }

    impl MemoryStore {
        fn with(key: &str, value: &str) -> Self {
            let store = Self::default();
            store.values.lock().unwrap().insert(key.to_string(), value.to_string());
            store
        }

        fn failing() -> Self {
            Self { fails: true, ..Self::default() }
        }

        fn delaying_missing_private_key_reads() -> Self {
            Self {
                missing_private_key_read_delay: Some(Duration::from_millis(25)),
                ..Self::default()
            }
        }
    }

    impl GemSecureStore for MemoryStore {
        fn get(&self, key: String) -> Result<Option<String>, GemServiceError> {
            if self.fails {
                return Err(GemServiceError::Core { msg: "unreadable".to_string() });
            }
            let value = self.values.lock().unwrap().get(&key).cloned();
            if key == DEVICE_PRIVATE_KEY
                && value.is_none()
                && let Some(delay) = self.missing_private_key_read_delay
            {
                thread::sleep(delay);
            }
            Ok(value)
        }

        fn set(&self, key: String, value: String) -> Result<(), GemServiceError> {
            self.writes.fetch_add(1, Ordering::Relaxed);
            self.values.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn remove(&self, key: String) -> Result<(), GemServiceError> {
            self.values.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    #[test]
    fn test_key_pair_is_created_once_and_persisted() {
        let store = Arc::new(MemoryStore::default());
        let service = GemDeviceKeyService::new(store.clone());

        let created = service.key_pair().unwrap();

        assert_eq!(created.private_key, service.key_pair().unwrap().private_key);
        assert_eq!(store.values.lock().unwrap().get(DEVICE_PRIVATE_KEY), Some(&hex::encode(&created.private_key)));
        assert_eq!(GemDeviceKeyService::new(store).device_id().unwrap(), hex::encode(&created.public_key));
    }

    #[test]
    fn test_concurrent_first_use_creates_one_identity() {
        const CALLERS: usize = 8;

        let store = Arc::new(MemoryStore::delaying_missing_private_key_reads());
        let service = Arc::new(GemDeviceKeyService::new(store.clone()));
        let barrier = Arc::new(Barrier::new(CALLERS));
        let handles = (0..CALLERS)
            .map(|_| {
                let service = service.clone();
                let barrier = barrier.clone();
                thread::spawn(move || {
                    barrier.wait();
                    service.key_pair().unwrap()
                })
            })
            .collect::<Vec<_>>();

        let key_pairs = handles.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<_>>();
        let first = &key_pairs[0];
        assert!(
            key_pairs
                .iter()
                .all(|key_pair| key_pair.private_key == first.private_key && key_pair.public_key == first.public_key)
        );

        let stored_private_key = store.values.lock().unwrap().get(DEVICE_PRIVATE_KEY).cloned();
        assert!(stored_private_key.is_some_and(|value| value == hex::encode(&first.private_key)));
        assert_eq!(store.writes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_stored_private_key_keeps_the_device_identity() {
        let key_pair = generate_device_key_pair();
        let store = Arc::new(MemoryStore::with(DEVICE_PRIVATE_KEY, &hex::encode(&key_pair.private_key)));

        let service = GemDeviceKeyService::new(store.clone());

        assert_eq!(service.device_id().unwrap(), hex::encode(&key_pair.public_key));
        assert_eq!(store.values.lock().unwrap().get(DEVICE_PUBLIC_KEY), Some(&hex::encode(&key_pair.public_key)));
    }

    #[test]
    fn test_unreadable_store_never_creates_a_new_identity() {
        let service = GemDeviceKeyService::new(Arc::new(MemoryStore::failing()));

        assert!(service.key_pair().is_err());
    }

    #[test]
    fn test_corrupt_private_key_never_creates_a_new_identity() {
        let service = GemDeviceKeyService::new(Arc::new(MemoryStore::with(DEVICE_PRIVATE_KEY, "not hex")));

        assert!(service.key_pair().is_err());
    }
}
