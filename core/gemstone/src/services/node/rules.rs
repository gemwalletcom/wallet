use std::collections::HashSet;

use primitives::Chain;
use primitives::node::{Node, NodeState};
use primitives::node_config::{self, NodePriority, NodeRegion};

pub fn merge_nodes(default_nodes: Vec<Node>, stored_nodes: Vec<Node>) -> Vec<Node> {
    let mut seen: HashSet<String> = HashSet::new();
    default_nodes.into_iter().chain(stored_nodes).filter(|node| seen.insert(node.url.clone())).collect()
}

pub fn is_default_node(url: &str, default_nodes: &[Node]) -> bool {
    default_nodes.iter().any(|node| node.url == url)
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
