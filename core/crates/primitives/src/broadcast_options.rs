use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastOptions {
    pub skip_preflight: bool,
}

impl BroadcastOptions {
    pub fn new(skip_preflight: bool) -> Self {
        Self { skip_preflight }
    }
}
