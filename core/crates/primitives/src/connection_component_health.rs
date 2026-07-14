use crate::ConnectionComponentMetadata;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
pub struct ConnectionComponentHealth {
    pub is_healthy: bool,
    pub metadata: Option<ConnectionComponentMetadata>,
}
