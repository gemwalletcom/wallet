pub mod model;
pub(crate) mod rules;

use std::{collections::HashMap, sync::Arc, time::Instant};

use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::services::stream::GemStreamConnection;

pub use model::{GemLatencyStatus, GemServiceStatusSession, GemServiceStatusTarget};

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

    pub fn new_session(&self) -> GemServiceStatusSession {
        GemServiceStatusSession::default()
    }

    pub async fn status(&self, target: GemServiceStatusTarget) -> GemLatencyStatus {
        match target {
            GemServiceStatusTarget::Endpoint { host } => self.endpoint_status(&host).await,
            GemServiceStatusTarget::Stream => self.stream.latency().await.into(),
        }
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
    use std::time::Duration;

    use futures::executor::block_on;
    use primitives::{GEM_API_HOST, node_config::NodeRegion};

    use super::*;
    use crate::models::list::GemListRow;
    use crate::services::stream::testkit::MemoryStreamConnection;
    use crate::testkit::TestAlienProvider;

    fn api() -> GemServiceStatusTarget {
        GemServiceStatusTarget::Endpoint { host: GEM_API_HOST.into() }
    }

    #[test]
    fn test_status_reports_reachable_endpoints_and_stream_failure_independently() {
        let service = GemServiceStatus::new(Arc::new(TestAlienProvider::with_status(200)), Arc::new(MemoryStreamConnection::default()));
        assert!(matches!(block_on(service.status(api())), GemLatencyStatus::Result { .. }));
        assert_eq!(block_on(service.status(GemServiceStatusTarget::Stream)), GemLatencyStatus::Error);
    }

    #[test]
    fn test_status_reports_http_failure_and_stream_latency_independently() {
        let stream = Arc::new(MemoryStreamConnection::default());
        *stream.latency.lock().unwrap() = Some(Duration::from_millis(125));
        let service = GemServiceStatus::new(Arc::new(TestAlienProvider::offline()), stream);
        assert_eq!(block_on(service.status(api())), GemLatencyStatus::Error);
        assert_eq!(block_on(service.status(GemServiceStatusTarget::Stream)), GemLatencyStatus::from(Some(Duration::from_millis(125))));
    }

    #[test]
    fn test_a_finished_check_shows_while_the_others_keep_loading() {
        let session = GemServiceStatusSession::default().on_status(GemServiceStatusTarget::Endpoint { host: NodeRegion::Asia.host().into() }, GemLatencyStatus::Error);
        let statuses: Vec<GemLatencyStatus> = session
            .sections()
            .into_iter()
            .flat_map(|section| section.rows)
            .map(|row| match row {
                GemListRow::Latency { status, .. } => status,
                _ => panic!("status rows are latency rows"),
            })
            .collect();
        assert_eq!(
            statuses,
            vec![GemLatencyStatus::Loading, GemLatencyStatus::Loading, GemLatencyStatus::Loading, GemLatencyStatus::Error, GemLatencyStatus::Loading]
        );
        assert_eq!(session.on_status(GemServiceStatusTarget::Stream, GemLatencyStatus::Error).stream, GemLatencyStatus::Error);
    }

    #[test]
    fn test_targets_cover_every_row() {
        let targets = GemServiceStatusSession::default().targets();
        assert_eq!(targets.first(), Some(&api()));
        assert_eq!(targets.last(), Some(&GemServiceStatusTarget::Stream));
        assert_eq!(targets.len(), NodeRegion::all().len() + 2);
    }
}
