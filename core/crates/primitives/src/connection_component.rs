use crate::ConnectionStatus;
use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Model)]
#[serde(rename_all = "camelCase")]
#[model(swift = "Equatable, Sendable")]
pub enum ConnectionComponent {
    Internet,
    Stream,
}

impl ConnectionComponent {
    pub fn failure_status(&self) -> ConnectionStatus {
        match self {
            Self::Internet => ConnectionStatus::NoInternet,
            Self::Stream => ConnectionStatus::NoService,
        }
    }
}
