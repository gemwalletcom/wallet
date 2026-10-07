pub mod model;
pub mod rules;
pub mod session;
pub mod settings;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesStore;
use std::fmt;
use std::sync::Arc;

use primitives::Chain;
use primitives::node::{Node, NodeState};

pub use model::{GemAddNodeError, GemExplorerRow, GemNodeCheck, GemNodeRow, GemNodeRowTitle, GemNodeSelection, GemNodeStatusState, GemNodeSubtitle};
pub use settings::GemChainSettingsService;
pub use store::GemNodeStore;

const NODE: &str = "node";

#[derive(uniffi::Object)]
pub struct GemNodeService {
    store: Arc<dyn GemNodeStore>,
    preferences: Arc<dyn GemPreferencesStore>,
}

impl fmt::Debug for GemNodeService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemNodeService").finish_non_exhaustive()
    }
}

#[uniffi::export]
impl GemNodeService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemNodeStore>, preferences: Arc<dyn GemPreferencesStore>) -> Self {
        Self { store, preferences }
    }

    pub fn websocket_node_url(&self, chain: Chain) -> String {
        rules::websocket_url(&self.node_url(chain))
    }
}

impl GemNodeService {
    pub async fn select_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        let selected = rules::selected_node(chain, Some(url), self.store.get_nodes(chain).await?);
        self.set_selected_url(chain, selected.url)
    }

    pub async fn ensure_selected_nodes(&self) -> Result<(), GemServiceError> {
        for chain in Chain::all() {
            let Some(url) = self.selected_url(chain) else {
                continue;
            };
            let selected = rules::selected_node(chain, Some(url.clone()), self.store.get_nodes(chain).await?);
            if selected.url != url {
                self.set_selected_url(chain, selected.url)?;
            }
        }
        Ok(())
    }

    pub async fn add_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        if !rules::can_delete_node(chain, &url) {
            return Ok(());
        }
        self.store.add_node(chain, Node { url, status: NodeState::Active, priority: 0 }).await
    }

    pub async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        if rules::is_default_node(&url, &rules::default_nodes(chain)) {
            return Ok(());
        }
        if self.selected_url(chain).as_deref() == Some(url.as_str()) {
            self.set_selected_url(chain, rules::fallback_node(chain).url)?;
        }
        self.store.delete_node(chain, url).await
    }

    pub async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        Ok(rules::merge_nodes(rules::default_nodes(chain), self.store.get_nodes(chain).await?))
    }
}

impl GemNodeService {
    pub(crate) fn node_url(&self, chain: Chain) -> String {
        self.selected_url(chain).unwrap_or_else(|| rules::fallback_node(chain).url)
    }

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
    use primitives::node_config::NodeRegion;

    #[test]
    fn test_merge_nodes_keeps_defaults_first_and_dedupes() {
        let merged = merge_nodes(vec![Node::mock("a", 0), Node::mock("b", 0)], vec![Node::mock("b", 0), Node::mock("c", 0)]);
        assert_eq!(merged.iter().map(|node| node.url.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert!(is_default_node("a", &[Node::mock("a", 0)]));
        assert!(!is_default_node("c", &[Node::mock("a", 0)]));
    }

    #[test]
    fn test_an_unknown_node_selects_the_us_region_and_listing_writes_nothing() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryNodeStore::default());
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let service = GemNodeService::new(store, preferences.clone());

            assert_eq!(service.node_url(Chain::Ethereum), NodeRegion::Us.url(Chain::Ethereum));

            service.get_nodes(Chain::Ethereum).await.unwrap();
            assert_eq!(preferences.get(node_key(Chain::Ethereum)), None, "listing the nodes is a read");

            service.select_node(Chain::Ethereum, "https://unknown.example".into()).await.unwrap();
            assert_eq!(service.node_url(Chain::Ethereum), NodeRegion::Us.url(Chain::Ethereum));
        });
    }

    #[test]
    fn test_the_launch_check_replaces_only_a_selection_the_chain_no_longer_offers() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryNodeStore::default());
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let service = GemNodeService::new(store.clone(), preferences.clone());
            let eu_url = NodeRegion::Eu.url(Chain::Solana);
            service.add_node(Chain::Sui, "https://added.example".into()).await.unwrap();
            service.select_node(Chain::Sui, "https://added.example".into()).await.unwrap();
            service.select_node(Chain::Solana, eu_url.clone()).await.unwrap();
            preferences.set(node_key(Chain::Ethereum), "https://removed.example".to_string()).unwrap();

            service.ensure_selected_nodes().await.unwrap();

            assert_eq!(service.node_url(Chain::Ethereum), NodeRegion::Us.url(Chain::Ethereum), "a node an update removed");
            assert_eq!(service.node_url(Chain::Sui), "https://added.example");
            assert_eq!(service.node_url(Chain::Solana), eu_url);
            assert_eq!(preferences.get(node_key(Chain::Bitcoin)), None, "no selection stays no selection");
        });
    }

    struct SelectedDuringRead {
        nodes: MemoryNodeStore,
        preferences: Arc<MemoryPreferencesStore>,
        selection: (Chain, String),
    }

    #[async_trait::async_trait]
    impl GemNodeStore for SelectedDuringRead {
        async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError> {
            let (selected_chain, url) = &self.selection;
            if *selected_chain == chain {
                self.preferences.set(node_key(chain), url.clone())?;
            }
            self.nodes.get_nodes(chain).await
        }
        async fn add_node(&self, chain: Chain, node: Node) -> Result<(), GemServiceError> {
            self.nodes.add_node(chain, node).await
        }
        async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
            self.nodes.delete_node(chain, url).await
        }
    }

    #[test]
    fn test_the_launch_check_keeps_a_node_the_user_picks_while_it_reads() {
        futures::executor::block_on(async {
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let asia_url = NodeRegion::Asia.url(Chain::Sui);
            let store = Arc::new(SelectedDuringRead {
                nodes: MemoryNodeStore::default(),
                preferences: preferences.clone(),
                selection: (Chain::Sui, asia_url.clone()),
            });
            let service = GemNodeService::new(store, preferences.clone());
            preferences.set(node_key(Chain::Sui), NodeRegion::Eu.url(Chain::Sui)).unwrap();

            service.ensure_selected_nodes().await.unwrap();

            assert_eq!(service.node_url(Chain::Sui), asia_url);
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
