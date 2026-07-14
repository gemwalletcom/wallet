use crate::Chain;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub struct InternetConnectionMetadata {
    pub is_expensive: bool,
    pub is_constrained: bool,
    pub is_vpn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub struct NodesConnectionMetadata {
    pub unreachable_chains: Vec<Chain>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub enum ConnectionComponentMetadata {
    Internet(InternetConnectionMetadata),
    Nodes(NodesConnectionMetadata),
}
