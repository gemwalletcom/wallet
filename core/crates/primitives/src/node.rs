use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct Node {
    pub url: String,
    pub status: NodeState,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct ChainNode {
    pub chain: String,
    pub node: Node,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct ChainNodes {
    pub chain: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[derive(Default)]
pub enum NodeState {
    #[default]
    Active,
    Inactive,
}
