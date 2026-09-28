use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Sendable")]
pub struct Node {
    pub url: String,
    pub status: NodeState,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[derive(Default)]
pub enum NodeState {
    #[default]
    Active,
    Inactive,
}
