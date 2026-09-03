use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::Latency;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "state", content = "latency", rename_all = "camelCase")]
pub enum ServiceStatusState {
    Loading,
    Result(Latency),
    Error,
}
