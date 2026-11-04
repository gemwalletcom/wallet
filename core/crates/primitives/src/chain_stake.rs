use std::str::FromStr;

use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

use crate::Chain;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum StakeChain {
    Cosmos,
    Osmosis,
    Injective,
    Sei,
    Celestia,
    Ethereum,
    Solana,
    Sui,
    SmartChain,
    Monad,
    Tron,
    Aptos,
    HyperCore,
}

impl StakeChain {
    pub fn chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }

    pub fn from_chain(chain: Chain) -> Option<Self> {
        Self::from_str(chain.as_ref()).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use strum::IntoEnumIterator;

    use super::StakeChain;
    use crate::Chain;

    #[test]
    fn test_from_chain_rejects_a_chain_without_staking() {
        assert_eq!(StakeChain::from_chain(Chain::Ethereum), Some(StakeChain::Ethereum));
        assert_eq!(StakeChain::from_chain(Chain::Bitcoin), None);
        assert_eq!(StakeChain::from_chain(Chain::Xrp), None);
    }

    #[test]
    fn test_ethereum_min_stake_amount() {
        assert_eq!(Chain::Ethereum.stake_config().map(|config| config.min_stake_amount), Some(10u64.pow(16)));
    }

    #[test]
    fn test_stake_chains_are_the_chains_with_a_stake_config() {
        assert_eq!(StakeChain::iter().map(|stake_chain| stake_chain.chain()).collect::<BTreeSet<_>>(), Chain::stakeable().into_iter().collect());
    }
}
