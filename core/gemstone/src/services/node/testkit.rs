use std::sync::{Arc, Mutex};

use primitives::node::Node;
use primitives::{Chain, Latency};

use super::model::{GemNodeCheck, GemNodeSelection, GemNodeStatusState};
use super::{GemChainSettingsService, GemNodeService, GemNodeStore};
use crate::gateway::GemGateway;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider, YieldOnce};

#[derive(Default)]
pub struct MemoryNodeStore {
    pub nodes: Mutex<Vec<Node>>,
    pub yields_between_read_and_write: bool,
}

#[async_trait::async_trait]
impl GemNodeStore for MemoryNodeStore {
    async fn get_nodes(&self, _chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        let nodes = self.nodes.lock().unwrap().clone();
        if self.yields_between_read_and_write {
            YieldOnce::default().await;
        }
        Ok(nodes)
    }
    async fn add_node(&self, _chain: Chain, node: Node) -> Result<(), GemServiceError> {
        self.nodes.lock().unwrap().push(node);
        Ok(())
    }
    async fn delete_node(&self, _chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.lock().unwrap().retain(|node| node.url != url);
        Ok(())
    }
}

impl GemChainSettingsService {
    pub fn mock() -> Self {
        let preferences = Arc::new(MemoryPreferencesStore::default());
        let nodes = Arc::new(GemNodeService::new(Arc::new(MemoryNodeStore::default()), preferences.clone()));
        Self::new(
            nodes.clone(),
            Arc::new(GemExplorerService::mock()),
            Arc::new(GemGateway::new(Arc::new(TestAlienProvider::with_status(200)), nodes, preferences, Arc::new(EmptyPreferences))),
        )
    }
}

impl GemNodeService {
    pub fn mock() -> Self {
        Self::new(Arc::new(MemoryNodeStore::default()), Arc::new(MemoryPreferencesStore::default()))
    }
}

impl GemNodeSelection {
    pub fn mock(url: &str) -> Self {
        Self {
            url: url.to_string(),
            host: url.to_string(),
            is_selected: false,
            gem_node_flag: None,
        }
    }
}

impl GemNodeStatusState {
    pub fn mock_result(latest_block_number: u64) -> Self {
        Self::Result {
            latest_block_number: Some(latest_block_number),
            latency: Latency::from_milliseconds(10),
        }
    }
}

impl GemNodeCheck {
    pub fn mock() -> Self {
        Self {
            url: "https://node".to_string(),
            chain_id: None,
            latest_block_number: Some(1),
            is_in_sync: true,
            latency: Latency::from_milliseconds(10),
        }
    }
}
