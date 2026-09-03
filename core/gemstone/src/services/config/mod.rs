use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::ConfigResponse;

use crate::api::{GemApiClient, GemApiError};
use crate::services::preferences::GemPreferencesService;

type ConfigResult = Result<ConfigResponse, GemServiceError>;

#[derive(uniffi::Object)]
pub struct GemConfigService {
    api: Arc<GemApiClient>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemConfigService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { api, preferences }
    }

    pub async fn update_config(&self) -> ConfigResult {
        let config = self.api.client.get_config().await.map_err(GemApiError::from)?;
        self.preferences.set_config(&config)?;
        Ok(config)
    }
}

impl GemConfigService {
    pub async fn get_config(&self) -> ConfigResult {
        match self.preferences.get_config() {
            Some(config) => Ok(config),
            None => self.update_config().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use async_trait::async_trait;
    use primitives::{Chain, ConfigVersions, SwapConfig};
    use std::future::Future;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll};

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
            Arc::new(GemApiClient::new(provider)),
            Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))),
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
    fn a_dropped_update_leaves_the_next_caller_free_to_read_the_stored_config() {
        let provider = Arc::new(ConfigProvider::default());
        let service = service(provider.clone());
        futures::executor::block_on(service.update_config()).unwrap();

        {
            let mut update = Box::pin(service.update_config());
            let waker = futures::task::noop_waker();
            assert!(update.as_mut().poll(&mut Context::from_waker(&waker)).is_pending());
        }

        let cached = futures::executor::block_on(service.get_config()).unwrap();

        assert_eq!(cached.versions.fiat_on_ramp_assets, 0);
        assert_eq!(provider.requests.load(Ordering::SeqCst), 2);
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
