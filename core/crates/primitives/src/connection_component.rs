use crate::ConnectionStatus;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub enum ConnectionComponent {
    Internet,
    Api,
    Nodes,
    Stream,
}

impl ConnectionComponent {
    pub fn failure_status(&self) -> ConnectionStatus {
        match self {
            Self::Internet => ConnectionStatus::NoInternet,
            Self::Api | Self::Nodes | Self::Stream => ConnectionStatus::NoService,
        }
    }
}
