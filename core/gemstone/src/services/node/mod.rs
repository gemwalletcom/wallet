pub mod model;
pub mod rules;
pub mod settings;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesStore;
use std::sync::Arc;

use primitives::Chain;
use primitives::node::{Node, NodeState};
use primitives::node_config::NodeRegion;

pub use model::{GemAddNodeError, GemNodeCheck, GemNodeStatusState};
pub use settings::GemChainSettingsService;
pub use store::GemNodeStore;

const NODE: &str = "node";

#[derive(uniffi::Object)]
pub struct GemNodeService {
    store: Arc<dyn GemNodeStore>,
    preferences: Arc<dyn GemPreferencesStore>,
}

#[uniffi::export]
impl GemNodeService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemNodeStore>, preferences: Arc<dyn GemPreferencesStore>) -> Self {
        Self { store, preferences }
    }

    pub fn get_default_nodes(&self, chain: Chain) -> Vec<Node> {
        rules::default_nodes(chain)
    }

    pub async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        let stored = self.store.get_nodes(chain).await?;
        let nodes = rules::merge_nodes(rules::default_nodes(chain), stored);
        let selected_url = self.selected_url(chain);
        let selected = rules::selected_node(selected_url.clone(), nodes.clone(), rules::region_node(chain, NodeRegion::Us));
        if selected_url.as_deref() != Some(selected.url.as_str()) {
            self.set_selected_url(chain, selected.url)?;
        }
        Ok(nodes)
    }

    pub fn can_delete_node(&self, chain: Chain, url: String) -> bool {
        rules::can_delete_node(chain, &url)
    }

    pub fn sorted_nodes(&self, chain: Chain, nodes: Vec<Node>) -> Vec<Node> {
        rules::sorted_nodes(chain, nodes)
    }

    pub fn node_url(&self, chain: Chain) -> String {
        rules::preferred_chain_node(chain, self.selected_url(chain)).url
    }

    pub fn websocket_node_url(&self, chain: Chain) -> String {
        rules::websocket_url(&self.node_url(chain))
    }

    pub fn selected_node(&self, chain: Chain) -> Node {
        rules::preferred_chain_node(chain, self.selected_url(chain))
    }

    pub async fn select_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        let stored = self.store.get_nodes(chain).await?;
        let selected = rules::chain_node(chain, Some(url), stored);
        self.set_selected_url(chain, selected.url)
    }

    pub async fn add_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        if !rules::can_delete_node(chain, &url) {
            return Ok(());
        }
        self.store
            .add_node(
                chain,
                Node {
                    url,
                    status: NodeState::Active,
                    priority: 0,
                },
            )
            .await
    }

    pub async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        if rules::is_default_node(&url, &rules::default_nodes(chain)) {
            return Ok(());
        }
        if self.selected_url(chain).as_deref() == Some(url.as_str()) {
            self.set_selected_url(chain, rules::region_node(chain, NodeRegion::Us).url)?;
        }
        self.store.delete_node(chain, url).await
    }
}

impl GemNodeService {
    fn selected_url(&self, chain: Chain) -> Option<String> {
        self.preferences.get(node_key(chain))
    }

    fn set_selected_url(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.preferences.set(node_key(chain), url)
    }
}

fn node_key(chain: Chain) -> String {
    format!("{NODE}_{}", chain.as_ref())
}

#[cfg(test)]
mod tests {
    use super::rules::*;
    use super::testkit::MemoryNodeStore;
    use super::*;
    use crate::services::preferences::testkit::MemoryPreferencesStore;

    fn node(url: &str) -> Node {
        Node {
            url: url.to_string(),
            status: NodeState::Active,
            priority: 0,
        }
    }

    #[test]
    fn test_merge_nodes_keeps_defaults_first_and_dedupes() {
        let merged = merge_nodes(vec![node("a"), node("b")], vec![node("b"), node("c")]);
        assert_eq!(merged.iter().map(|node| node.url.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert!(is_default_node("a", &[node("a")]));
        assert!(!is_default_node("c", &[node("a")]));
    }

    #[test]
    fn test_selected_node_falls_back_to_us_region() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryNodeStore::default());
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let service = GemNodeService::new(store, preferences.clone());

            assert_eq!(service.node_url(Chain::Ethereum), NodeRegion::Us.url(Chain::Ethereum));

            service.get_nodes(Chain::Ethereum).await.unwrap();
            assert_eq!(preferences.get(node_key(Chain::Ethereum)), Some(NodeRegion::Us.url(Chain::Ethereum)));

            service.select_node(Chain::Ethereum, "https://unknown.example".into()).await.unwrap();
            assert_eq!(service.node_url(Chain::Ethereum), NodeRegion::Us.url(Chain::Ethereum));
        });
    }

    #[test]
    fn test_delete_node_keeps_defaults_and_selects_fallback() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryNodeStore::default());
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let service = GemNodeService::new(store.clone(), preferences.clone());
            let default_url = NodeRegion::Eu.url(Chain::Ethereum);

            service.add_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.select_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.delete_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.delete_node(Chain::Ethereum, default_url.clone()).await.unwrap();

            let fallback_url = NodeRegion::Us.url(Chain::Ethereum);
            assert_eq!(preferences.get(node_key(Chain::Ethereum)).as_deref(), Some(fallback_url.as_str()));
            assert!(!service.get_nodes(Chain::Ethereum).await.unwrap().iter().any(|node| node.url == "https://custom.example"));
            assert!(service.get_nodes(Chain::Ethereum).await.unwrap().iter().any(|node| node.url == default_url));
        });
    }

    #[test]
    fn test_node_url_uses_persisted_selection_and_falls_back_when_unset() {
        let preferences = Arc::new(MemoryPreferencesStore::default());
        let service = GemNodeService::new(Arc::new(MemoryNodeStore::default()), preferences.clone());
        let us_url = NodeRegion::Us.url(Chain::Ethereum);

        assert_eq!(service.node_url(Chain::Ethereum), us_url);
        preferences.set(node_key(Chain::Ethereum), "https://persisted.example".to_string()).unwrap();
        assert_eq!(service.node_url(Chain::Ethereum), "https://persisted.example");
        assert_eq!(node_key(Chain::Ethereum), "node_ethereum");
    }
}
