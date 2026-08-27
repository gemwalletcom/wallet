use std::collections::HashSet;

use primitives::node::Node;

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
