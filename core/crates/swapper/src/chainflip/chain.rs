use primitives::Chain;
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
pub enum ChainflipChain {
    Ethereum,
    Bitcoin,
    Solana,
    Arbitrum,
    Tron,
}

impl ChainflipChain {
    pub fn from_chain(chain: Chain) -> Option<Self> {
        match chain {
            Chain::Ethereum => Some(Self::Ethereum),
            Chain::Bitcoin => Some(Self::Bitcoin),
            Chain::Solana => Some(Self::Solana),
            Chain::Arbitrum => Some(Self::Arbitrum),
            Chain::Tron => Some(Self::Tron),
            _ => None,
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Ethereum => Chain::Ethereum,
            Self::Bitcoin => Chain::Bitcoin,
            Self::Solana => Chain::Solana,
            Self::Arbitrum => Chain::Arbitrum,
            Self::Tron => Chain::Tron,
        }
    }
}
