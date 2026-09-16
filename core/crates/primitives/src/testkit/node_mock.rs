use crate::{Node, NodeState};

impl Node {
    pub fn mock(url: &str, priority: i32) -> Self {
        Self {
            url: url.to_string(),
            status: NodeState::Active,
            priority,
        }
    }
}
