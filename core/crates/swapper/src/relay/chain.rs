use primitives::{Chain, ChainType, chain_evm::EVMChain};

const SOLANA_CHAIN_ID: u64 = 792703809;
pub(super) const TON_CHAIN_ID: u64 = 224235520;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayChain {
    Evm(EVMChain),
    Tron,
    Solana,
    Ton,
}

impl RelayChain {
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            Self::Evm(chain) => Some(chain.chain_id()),
            Self::Tron => Chain::Tron.network_id_value(),
            Self::Solana => Some(SOLANA_CHAIN_ID),
            Self::Ton => Some(TON_CHAIN_ID),
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<Self> {
        match chain.chain_type() {
            ChainType::Ethereum => Some(Self::Evm(EVMChain::from_chain(*chain)?)),
            ChainType::Tron => Some(Self::Tron),
            ChainType::Solana => Some(Self::Solana),
            ChainType::Ton => Some(Self::Ton),
            _ => None,
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Evm(chain) => chain.to_chain(),
            Self::Tron => Chain::Tron,
            Self::Solana => Chain::Solana,
            Self::Ton => Chain::Ton,
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            SOLANA_CHAIN_ID => Some(Self::Solana),
            TON_CHAIN_ID => Some(Self::Ton),
            _ => Self::from_chain(&Chain::from_chain_id(chain_id)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_chain() {
        assert_eq!(RelayChain::from_chain(&Chain::Ethereum).unwrap().chain_id(), Some(EVMChain::Ethereum.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::SmartChain).unwrap().chain_id(), Some(EVMChain::SmartChain.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::Robinhood).unwrap().chain_id(), Some(EVMChain::Robinhood.chain_id()));
        assert_eq!(RelayChain::from_chain(&Chain::Tron), Some(RelayChain::Tron));
        assert_eq!(RelayChain::Tron.chain_id(), Some(728126428));
        assert_eq!(RelayChain::from_chain_id(728126428), Some(RelayChain::Tron));
        assert_eq!(RelayChain::Tron.to_chain(), Chain::Tron);
        assert_eq!(RelayChain::from_chain(&Chain::Solana), Some(RelayChain::Solana));
        assert_eq!(RelayChain::Solana.chain_id(), Some(792703809));
        assert_eq!(RelayChain::from_chain_id(792703809), Some(RelayChain::Solana));
        assert_eq!(RelayChain::Solana.to_chain(), Chain::Solana);
        assert_eq!(RelayChain::from_chain(&Chain::Ton), Some(RelayChain::Ton));
        assert_eq!(RelayChain::Ton.chain_id(), Some(224235520));
        assert_eq!(RelayChain::from_chain_id(224235520), Some(RelayChain::Ton));
        assert_eq!(RelayChain::Ton.to_chain(), Chain::Ton);
        assert!(RelayChain::from_chain(&Chain::Bitcoin).is_none());
        assert!(RelayChain::from_chain(&Chain::Cosmos).is_none());
    }
}
