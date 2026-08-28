use crate::services::collections::unique_by;
use primitives::Chain;
use primitives::node::{Node, NodeState};
use primitives::node_config::{self, NodePriority, NodeRegion};

pub fn merge_nodes(default_nodes: Vec<Node>, stored_nodes: Vec<Node>) -> Vec<Node> {
    unique_by(default_nodes.into_iter().chain(stored_nodes), |node| node.url.clone())
}

pub fn is_default_node(url: &str, default_nodes: &[Node]) -> bool {
    default_nodes.iter().any(|node| node.url == url)
}

pub fn can_delete_node(chain: Chain, url: &str) -> bool {
    !is_default_node(url, &default_nodes(chain))
}

pub fn sorted_nodes(chain: Chain, nodes: Vec<Node>) -> Vec<Node> {
    let defaults = default_nodes(chain);
    let (default_nodes, added): (Vec<Node>, Vec<Node>) = nodes.into_iter().partition(|node| is_default_node(&node.url, &defaults));
    default_nodes.into_iter().chain(added).collect()
}

pub fn selected_node(selected_url: Option<String>, nodes: Vec<Node>, fallback: Node) -> Node {
    selected_url.and_then(|url| nodes.into_iter().find(|node| node.url == url)).unwrap_or(fallback)
}

pub fn chain_node(chain: Chain, selected_url: Option<String>, stored_nodes: Vec<Node>) -> Node {
    selected_node(selected_url, merge_nodes(default_nodes(chain), stored_nodes), region_node(chain, NodeRegion::Us))
}

pub fn region_node(chain: Chain, region: NodeRegion) -> Node {
    Node {
        url: region.url(chain),
        status: NodeState::Active,
        priority: region.priority(),
    }
}

pub fn config_node(node: node_config::Node) -> Node {
    let (status, priority) = match node.priority {
        NodePriority::High => (NodeState::Active, 3),
        NodePriority::Medium => (NodeState::Active, 2),
        NodePriority::Low => (NodeState::Active, 1),
        NodePriority::Inactive => (NodeState::Inactive, 0),
    };
    Node { url: node.url, status, priority }
}

pub fn default_nodes(chain: Chain) -> Vec<Node> {
    NodeRegion::all()
        .into_iter()
        .map(|region| region_node(chain, region))
        .chain(node_config::get_nodes_for_chain(chain).into_iter().map(config_node))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(url: &str, priority: i32) -> Node {
        Node {
            url: url.to_string(),
            status: NodeState::Active,
            priority,
        }
    }

    #[test]
    fn test_merge_nodes_keeps_defaults_first_and_drops_duplicate_urls() {
        let merged = merge_nodes(vec![node("https://a", 1)], vec![node("https://a", 5), node("https://b", 2)]);

        assert_eq!(merged.iter().map(|node| node.url.as_str()).collect::<Vec<_>>(), vec!["https://a", "https://b"]);
        assert_eq!(merged[0].priority, 1);
        assert!(is_default_node("https://a", &[node("https://a", 1)]));
        assert!(!is_default_node("https://b", &[node("https://a", 1)]));
    }

    #[test]
    fn test_selected_node_falls_back_when_url_is_missing() {
        let nodes = vec![node("https://a", 1), node("https://b", 2)];

        assert_eq!(selected_node(Some("https://b".to_string()), nodes.clone(), node("https://f", 0)).url, "https://b");
        assert_eq!(selected_node(Some("https://c".to_string()), nodes.clone(), node("https://f", 0)).url, "https://f");
        assert_eq!(selected_node(None, nodes, node("https://f", 0)).url, "https://f");
    }

    #[test]
    fn test_chain_node_defaults_to_us_region() {
        let chain = Chain::Ethereum;

        assert_eq!(chain_node(chain, None, vec![]).url, NodeRegion::Us.url(chain));
        assert_eq!(chain_node(chain, Some("https://custom".to_string()), vec![node("https://custom", 1)]).url, "https://custom");
        assert!(default_nodes(chain).iter().any(|node| node.url == NodeRegion::Eu.url(chain)));
    }

    #[test]
    fn test_config_node_maps_priority_to_state() {
        let config = |priority: NodePriority| {
            config_node(node_config::Node {
                url: "https://n".to_string(),
                priority,
            })
        };

        assert_eq!((config(NodePriority::High).status, config(NodePriority::High).priority), (NodeState::Active, 3));
        assert_eq!((config(NodePriority::Low).status, config(NodePriority::Low).priority), (NodeState::Active, 1));
        assert_eq!((config(NodePriority::Inactive).status, config(NodePriority::Inactive).priority), (NodeState::Inactive, 0));
    }

    #[test]
    fn test_only_added_nodes_can_be_deleted_and_defaults_sort_first() {
        let default_url = NodeRegion::Us.url(Chain::Ethereum);
        let added = node("https://added.example", 1);

        assert!(!can_delete_node(Chain::Ethereum, &default_url));
        assert!(can_delete_node(Chain::Ethereum, &added.url));

        let sorted = sorted_nodes(Chain::Ethereum, vec![added.clone(), node(&default_url, 3)]);
        assert_eq!(
            sorted.iter().map(|node| node.url.as_str()).collect::<Vec<_>>(),
            vec![default_url.as_str(), added.url.as_str()]
        );
    }
}
