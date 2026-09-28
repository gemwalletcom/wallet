use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::Chain;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>),
}

impl SwapProviderMode {
    pub fn quote_lifetime(&self) -> Duration {
        match self {
            Self::OnChain => Duration::from_secs(60),
            Self::CrossChain | Self::Bridge | Self::OmniChain(_) => Duration::from_secs(300),
        }
    }
}
