use std::str::FromStr;

use gem_polkadot::PolkadotAddress;
use gem_solana::Pubkey;
use gem_tron::address::TronAddress;
use primitives::{Address, Chain};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
pub enum ChainflipChain {
    Ethereum,
    Bitcoin,
    Assethub,
    Solana,
    Arbitrum,
    Tron,
    Bsc,
}

impl ChainflipChain {
    pub fn from_chain(chain: Chain) -> Option<Self> {
        match chain {
            Chain::Ethereum => Some(Self::Ethereum),
            Chain::Bitcoin => Some(Self::Bitcoin),
            Chain::Polkadot => Some(Self::Assethub),
            Chain::Solana => Some(Self::Solana),
            Chain::Arbitrum => Some(Self::Arbitrum),
            Chain::Tron => Some(Self::Tron),
            Chain::SmartChain => Some(Self::Bsc),
            _ => None,
        }
    }

    pub fn id(self) -> u32 {
        match self {
            Self::Ethereum => 1,
            Self::Bitcoin => 3,
            Self::Arbitrum => 4,
            Self::Solana => 5,
            Self::Assethub => 6,
            Self::Tron => 7,
            Self::Bsc => 8,
        }
    }

    pub fn scale_address_variant(self) -> u8 {
        match self {
            Self::Ethereum => 0,
            Self::Bitcoin => 2,
            Self::Arbitrum => 3,
            Self::Solana => 4,
            Self::Assethub => 5,
            Self::Tron => 6,
            Self::Bsc => 7,
        }
    }

    pub fn encode_address(self, address: &str) -> Option<Vec<u8>> {
        match self {
            Self::Ethereum | Self::Arbitrum | Self::Bsc => alloy_primitives::Address::from_str(address).ok().map(|address| address.to_vec()),
            Self::Bitcoin => Some(address.as_bytes().to_vec()),
            Self::Solana => Pubkey::from_str(address).ok().map(|address| address.as_bytes().to_vec()),
            Self::Assethub => PolkadotAddress::try_parse(address).map(|address| address.as_bytes().to_vec()),
            Self::Tron => TronAddress::parse(address).ok().map(|address| alloy_primitives::Address::from(address).to_vec()),
        }
    }

    pub fn to_chain(self) -> Chain {
        match self {
            Self::Ethereum => Chain::Ethereum,
            Self::Bitcoin => Chain::Bitcoin,
            Self::Assethub => Chain::Polkadot,
            Self::Solana => Chain::Solana,
            Self::Arbitrum => Chain::Arbitrum,
            Self::Tron => Chain::Tron,
            Self::Bsc => Chain::SmartChain,
        }
    }
}
