pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::Chain;
use primitives::node::{Node, NodeState};

pub use store::GemNodeStore;

#[derive(uniffi::Object)]
pub struct GemNodeService {
    store: Arc<dyn GemNodeStore>,
}

#[uniffi::export]
impl GemNodeService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemNodeStore>) -> Self {
        Self { store }
    }

    pub fn get_default_nodes(&self, chain: Chain) -> Vec<Node> {
        rules::default_nodes(chain)
    }

    pub async fn get_nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        let stored = self.store.get_nodes(chain).await?;
        Ok(rules::merge_nodes(rules::default_nodes(chain), stored))
    }

    pub fn node_url(&self, chain: Chain, selected_url: Option<String>, stored_nodes: Vec<Node>) -> String {
        rules::chain_node(chain, selected_url, stored_nodes).url
    }

    pub fn selected_node(&self, chain: Chain, selected_url: Option<String>, stored_nodes: Vec<Node>) -> Node {
        rules::chain_node(chain, selected_url, stored_nodes)
    }

    pub async fn set_selected_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.store.set_selected_url(chain, url).await
    }

    pub async fn add_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
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
        self.store.delete_node(chain, url.clone()).await?;
        if self.store.get_selected_url(chain).await? == Some(url) {
            self.store.delete_selected_url(chain).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::rules::*;
    use super::*;
    use primitives::node_config::NodeRegion;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        nodes: Mutex<Vec<Node>>,
        selected: Mutex<Option<String>>,
    }

    #[async_trait::async_trait]
    impl GemNodeStore for MemoryStore {
        async fn get_nodes(&self, _chain: Chain) -> Result<Vec<Node>, GemServiceError> {
            Ok(self.nodes.lock().unwrap().clone())
        }
        async fn add_node(&self, _chain: Chain, node: Node) -> Result<(), GemServiceError> {
            self.nodes.lock().unwrap().push(node);
            Ok(())
        }
        async fn delete_node(&self, _chain: Chain, url: String) -> Result<(), GemServiceError> {
            self.nodes.lock().unwrap().retain(|node| node.url != url);
            Ok(())
        }
        async fn get_selected_url(&self, _chain: Chain) -> Result<Option<String>, GemServiceError> {
            Ok(self.selected.lock().unwrap().clone())
        }
        async fn set_selected_url(&self, _chain: Chain, url: String) -> Result<(), GemServiceError> {
            *self.selected.lock().unwrap() = Some(url);
            Ok(())
        }
        async fn delete_selected_url(&self, _chain: Chain) -> Result<(), GemServiceError> {
            *self.selected.lock().unwrap() = None;
            Ok(())
        }
    }

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
            let store = Arc::new(MemoryStore::default());
            let service = GemNodeService::new(store.clone());

            let node_url = |service: &GemNodeService| service.node_url(Chain::Ethereum, None, Vec::new());
            assert_eq!(node_url(&service), NodeRegion::Us.url(Chain::Ethereum));

            service.set_selected_node(Chain::Ethereum, "https://unknown.example".into()).await.unwrap();
            let selected_url = store.get_selected_url(Chain::Ethereum).await.unwrap();
            assert_eq!(service.node_url(Chain::Ethereum, selected_url, Vec::new()), NodeRegion::Us.url(Chain::Ethereum));
        });
    }

    #[test]
    fn test_delete_node_keeps_defaults_and_clears_selection() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryStore::default());
            let service = GemNodeService::new(store.clone());
            let default_url = NodeRegion::Eu.url(Chain::Ethereum);

            service.add_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.set_selected_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.delete_node(Chain::Ethereum, "https://custom.example".into()).await.unwrap();
            service.delete_node(Chain::Ethereum, default_url.clone()).await.unwrap();

            assert!(store.selected.lock().unwrap().is_none());
            assert!(!service.get_nodes(Chain::Ethereum).await.unwrap().iter().any(|node| node.url == "https://custom.example"));
            assert!(service.get_nodes(Chain::Ethereum).await.unwrap().iter().any(|node| node.url == default_url));
        });
    }

    #[test]
    fn test_node_url_uses_known_selection_and_falls_back_to_us_region() {
        let service = GemNodeService::new(Arc::new(MemoryStore::default()));
        let custom = Node {
            url: "https://custom.example".to_string(),
            status: NodeState::Active,
            priority: 0,
        };
        let us_url = NodeRegion::Us.url(Chain::Ethereum);

        assert_eq!(service.node_url(Chain::Ethereum, Some(custom.url.clone()), vec![custom.clone()]), custom.url);
        assert_eq!(service.node_url(Chain::Ethereum, Some("https://unknown.example".to_string()), vec![custom]), us_url);
        assert_eq!(service.node_url(Chain::Ethereum, None, vec![]), us_url);
    }
}
