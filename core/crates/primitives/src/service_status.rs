use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "state", content = "latencyMilliseconds", rename_all = "camelCase")]
pub enum ServiceStatusState {
    Loading,
    Result(UInt64),
    Error,
}
