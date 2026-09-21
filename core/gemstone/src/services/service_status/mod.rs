pub mod model;
pub(crate) mod rules;

use std::{
    collections::HashMap,
    iter::once,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::future::join_all;
use primitives::{GEM_API_HOST, node_config::NodeRegion};

use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::models::list::GemListSection;
use crate::services::stream::GemStreamConnection;

pub use model::GemLatencyStatus;

#[uniffi::export]
pub fn service_status_timeout() -> Duration {
    gem_client::DEFAULT_REQUEST_TIMEOUT
}

#[derive(uniffi::Object)]
pub struct GemServiceStatus {
    provider: Arc<dyn AlienProvider>,
    stream: Arc<dyn GemStreamConnection>,
}

#[uniffi::export]
impl GemServiceStatus {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, stream: Arc<dyn GemStreamConnection>) -> Self {
        Self { provider, stream }
    }

    pub fn sections(&self) -> Vec<GemListSection> {
        rules::sections(&HashMap::new(), GemLatencyStatus::Loading)
    }

    pub async fn load(&self) -> Vec<GemListSection> {
        let hosts = once(GEM_API_HOST).chain(NodeRegion::all().into_iter().map(|region| region.host()));
        let requests = hosts.map(|host| async move { (host.to_string(), self.endpoint_status(host).await) });
        let (statuses, stream) = futures::join!(join_all(requests), self.stream.latency());
        rules::sections(&statuses.into_iter().collect(), stream.into())
    }
}

impl GemServiceStatus {
    async fn endpoint_status(&self, host: &str) -> GemLatencyStatus {
        let target = AlienTarget {
            url: format!("https://{host}"),
            method: AlienHttpMethod::Get,
            headers: Some(HashMap::from([("Cache-Control".to_string(), "no-cache".to_string())])),
            body: None,
        };
        let start = Instant::now();
        self.provider.request(target).await.ok().map(|_| start.elapsed()).into()
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;
    use crate::models::list::GemListRow;
    use crate::services::stream::testkit::MemoryStreamConnection;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_load_reports_reachable_endpoints_and_stream_failure_independently() {
        let service = GemServiceStatus::new(Arc::new(TestAlienProvider::with_status(200)), Arc::new(MemoryStreamConnection::default()));
        let sections = block_on(service.load());
        assert!(matches!(sections[0].rows[0], GemListRow::Latency { status: GemLatencyStatus::Result { .. }, .. }));
        assert!(matches!(sections[0].rows[1], GemListRow::Latency { status: GemLatencyStatus::Error, .. }));
        assert!(sections[1].rows.iter().all(|row| matches!(row, GemListRow::Latency { status: GemLatencyStatus::Result { .. }, .. })));
    }

    #[test]
    fn test_load_reports_http_failure_and_stream_latency_independently() {
        let stream = Arc::new(MemoryStreamConnection::default());
        *stream.latency.lock().unwrap() = Some(Duration::from_millis(125));
        let service = GemServiceStatus::new(Arc::new(TestAlienProvider::offline()), stream);
        let sections = block_on(service.load());
        assert!(matches!(sections[0].rows[0], GemListRow::Latency { status: GemLatencyStatus::Error, .. }));
        assert!(matches!(&sections[0].rows[1], GemListRow::Latency { status, .. } if *status == GemLatencyStatus::from(Some(Duration::from_millis(125)))));
    }
}
