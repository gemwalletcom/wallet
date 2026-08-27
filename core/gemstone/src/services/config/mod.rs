use crate::services::error::GemServiceError;
use futures::channel::oneshot;
use std::sync::{Arc, Mutex, MutexGuard};

use primitives::ConfigResponse;

use crate::api::{GemApiClient, GemApiError};
use crate::services::preferences::GemPreferencesService;

type ConfigResult = Result<ConfigResponse, GemServiceError>;
type Waiters = Option<Vec<oneshot::Sender<ConfigResult>>>;

#[derive(uniffi::Object)]
pub struct GemConfigService {
    api: Arc<GemApiClient>,
    preferences: Arc<GemPreferencesService>,
    waiters: Mutex<Waiters>,
}

#[uniffi::export]
impl GemConfigService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, preferences: Arc<GemPreferencesService>) -> Self {
        Self {
            api,
            preferences,
            waiters: Mutex::new(None),
        }
    }

    pub async fn update_config(&self) -> ConfigResult {
        let receiver = {
            let mut waiters = self.waiters();
            match waiters.as_mut() {
                Some(pending) => {
                    let (sender, receiver) = oneshot::channel();
                    pending.push(sender);
                    Some(receiver)
                }
                None => {
                    *waiters = Some(Vec::new());
                    None
                }
            }
        };
        if let Some(receiver) = receiver {
            return receiver.await.unwrap_or_else(|_| {
                Err(GemServiceError::Status {
                    msg: "config update cancelled".to_string(),
                })
            });
        }
        let result = self.load_config().await;
        for sender in self.waiters().take().unwrap_or_default() {
            let _ = sender.send(result.clone());
        }
        result
    }
}

impl GemConfigService {
    pub async fn get_config(&self) -> ConfigResult {
        if self.waiters().is_some() {
            return self.update_config().await;
        }
        match self.preferences.get_config() {
            Some(config) => Ok(config),
            None => self.update_config().await,
        }
    }
}

impl GemConfigService {
    async fn load_config(&self) -> ConfigResult {
        let config = self.api.client.get_config().await.map_err(GemApiError::from)?;
        self.preferences.set_config(&config)?;
        Ok(config)
    }

    fn waiters(&self) -> MutexGuard<'_, Waiters> {
        match self.waiters.lock() {
            Ok(waiters) => waiters,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
    use crate::services::preferences::GemPreferencesStore;
    use async_trait::async_trait;
    use primitives::{Chain, ConfigVersions, SwapConfig};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::Poll;

    #[derive(Default)]
    struct MemoryStore {
        values: Mutex<HashMap<String, String>>,
    }

    impl GemPreferencesStore for MemoryStore {
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
    }

    #[derive(Debug, Default)]
    struct ConfigProvider {
        requests: AtomicUsize,
    }

    #[async_trait]
    impl AlienProvider for ConfigProvider {
        async fn request(&self, _target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
            let request = self.requests.fetch_add(1, Ordering::SeqCst);
            let mut yielded = false;
            futures::future::poll_fn(|context| {
                if yielded {
                    Poll::Ready(())
                } else {
                    yielded = true;
                    context.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await;
            let config = ConfigResponse {
                releases: vec![],
                versions: ConfigVersions {
                    fiat_on_ramp_assets: request as i32,
                    fiat_off_ramp_assets: 0,
                    swap_assets: 0,
                },
                swap: SwapConfig { enabled_providers: vec![] },
            };
            Ok(Arc::new(AlienResponse::new(Some(200), serde_json::to_vec(&config).unwrap())))
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok("https://example.com".to_string())
        }
    }

    fn service(provider: Arc<ConfigProvider>) -> GemConfigService {
        GemConfigService::new(
            Arc::new(GemApiClient::new(provider, "https://example.com".to_string())),
            Arc::new(GemPreferencesService::new(Arc::new(MemoryStore::default()))),
        )
    }

    #[test]
    fn concurrent_updates_share_one_request() {
        let provider = Arc::new(ConfigProvider::default());
        let service = service(provider.clone());

        let (first, second, cached) = futures::executor::block_on(futures::future::join3(service.update_config(), service.update_config(), service.get_config()));

        assert_eq!(provider.requests.load(Ordering::SeqCst), 1);
        assert_eq!(first.unwrap().versions.fiat_on_ramp_assets, 0);
        assert_eq!(second.unwrap().versions.fiat_on_ramp_assets, 0);
        assert_eq!(cached.unwrap().versions.fiat_on_ramp_assets, 0);
    }

    #[test]
    fn get_config_uses_cache_after_update() {
        let provider = Arc::new(ConfigProvider::default());
        let service = service(provider.clone());

        futures::executor::block_on(async {
            service.update_config().await.unwrap();
            service.get_config().await.unwrap();
            service.update_config().await.unwrap();
        });

        assert_eq!(provider.requests.load(Ordering::SeqCst), 2);
    }
}
