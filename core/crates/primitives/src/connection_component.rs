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
