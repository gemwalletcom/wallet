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

    pub fn first_quote_wait(&self) -> Duration {
        match self {
            Self::OnChain => Duration::from_millis(500),
            Self::CrossChain | Self::Bridge | Self::OmniChain(_) => Duration::from_secs(1),
        }
    }

    pub fn quote_timeout(&self) -> Duration {
        match self {
            Self::OnChain => Duration::from_secs(8),
            Self::CrossChain | Self::Bridge | Self::OmniChain(_) => Duration::from_secs(15),
        }
    }

    pub fn for_pair(from_chain: Chain, to_chain: Chain) -> Self {
        if from_chain == to_chain { Self::OnChain } else { Self::CrossChain }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_first_quote_wait_and_the_timeout_follow_the_pair() {
        assert_eq!(SwapProviderMode::for_pair(Chain::Ethereum, Chain::Ethereum), SwapProviderMode::OnChain);
        assert_eq!(SwapProviderMode::for_pair(Chain::Arbitrum, Chain::Solana), SwapProviderMode::CrossChain);
        assert_eq!((SwapProviderMode::OnChain.first_quote_wait(), SwapProviderMode::OnChain.quote_timeout()), (Duration::from_millis(500), Duration::from_secs(8)));
        for mode in [SwapProviderMode::CrossChain, SwapProviderMode::Bridge, SwapProviderMode::OmniChain(vec![Chain::Near])] {
            assert_eq!((mode.first_quote_wait(), mode.quote_timeout()), (Duration::from_secs(1), Duration::from_secs(15)));
        }
    }
}
