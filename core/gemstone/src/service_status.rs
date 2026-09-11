use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use primitives::{GEM_API_HOST, Latency, node_config::NodeRegion};

use crate::GemstoneError;
use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};

#[uniffi::export]
pub fn service_status_timeout() -> Duration {
    gem_client::DEFAULT_REQUEST_TIMEOUT
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum GemServiceEndpointType {
    Api,
    GemNode,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemServiceEndpoint {
    pub endpoint_type: GemServiceEndpointType,
    pub host: String,
    pub url: String,
    pub flag: String,
}

impl GemServiceEndpoint {
    fn new(endpoint_type: GemServiceEndpointType, host: &str, flag: &str) -> Self {
        Self {
            endpoint_type,
            host: host.to_string(),
            url: format!("https://{host}"),
            flag: flag.to_string(),
        }
    }
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum GemLatencyStatus {
    Loading,
    Error,
    Result { latency: Latency },
}

#[derive(uniffi::Object)]
pub struct GemServiceStatus {
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl GemServiceStatus {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub fn get_endpoints(&self) -> Vec<GemServiceEndpoint> {
        [GemServiceEndpoint::new(GemServiceEndpointType::Api, GEM_API_HOST, NodeRegion::Us.flag())]
            .into_iter()
            .chain(
                NodeRegion::all()
                    .into_iter()
                    .map(|region| GemServiceEndpoint::new(GemServiceEndpointType::GemNode, region.host(), region.flag())),
            )
            .collect()
    }

    pub async fn get_endpoint_status(&self, url: String) -> GemLatencyStatus {
        match self.get_endpoint_latency(url).await {
            Ok(latency) => GemLatencyStatus::Result { latency },
            Err(_) => GemLatencyStatus::Error,
        }
    }
}

impl GemServiceStatus {
    async fn get_endpoint_latency(&self, url: String) -> Result<Latency, GemstoneError> {
        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: Some(HashMap::from([("Cache-Control".to_string(), "no-cache".to_string())])),
            body: None,
        };
        let start_time = Instant::now();
        self.provider.request(target).await?;

        Ok(Latency::from_milliseconds(start_time.elapsed().as_millis() as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::{AlienError, AlienResponse};
    use crate::testkit::TestAlienProvider;
    use async_trait::async_trait;
    use futures::executor::block_on;
    use primitives::LatencyType;

    #[derive(Debug)]
    struct OfflineProvider;

    #[async_trait]
    impl AlienProvider for OfflineProvider {
        async fn request(&self, _target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
            Err(AlienError::Offline)
        }
    }

    #[test]
    fn test_endpoint_status_measures_a_reachable_endpoint_and_reports_an_unreachable_one() {
        let reachable = GemServiceStatus::new(Arc::new(TestAlienProvider::with_status(200)));
        match block_on(reachable.get_endpoint_status("https://api.gemwallet.com".to_string())) {
            GemLatencyStatus::Result { latency } => assert_eq!(latency.latency_type, LatencyType::Fast),
            other => panic!("a reachable endpoint reports its latency, got {other:?}"),
        }

        let unreachable = GemServiceStatus::new(Arc::new(OfflineProvider));
        assert_eq!(block_on(unreachable.get_endpoint_status("https://api.gemwallet.com".to_string())), GemLatencyStatus::Error);
    }
}
