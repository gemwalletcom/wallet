mod asset;
mod chain;
pub mod client;
mod constants;
mod contracts;
pub mod memo;
pub mod model;
mod provider;
mod quote_data_mapper;
mod quote_mapper;
mod swap_mapper;

pub use provider::ThorChain;

use strum::Display;

use super::SwapperProvider;

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const DUST_THRESHOLD_MULTIPLIER: i64 = 2;

// FIXME: estimate gas limit with memo x bytes
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 90_000;

#[derive(Debug, Clone, Copy, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum THORChainNetwork {
    Thorchain,
    Mayachain,
}

impl THORChainNetwork {
    pub fn provider(&self) -> SwapperProvider {
        match self {
            Self::Thorchain => SwapperProvider::Thorchain,
            Self::Mayachain => SwapperProvider::Mayachain,
        }
    }

    pub fn router_addresses(&self) -> &'static [&'static str] {
        match self {
            Self::Thorchain => &[
                "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146", // Ethereum
                "0xb30eC53F98ff5947EDe720D32aC2da7e52A5f56b", // SmartChain
                "0x8F66c4AE756BEbC49Ec8B81966DD8bba9f127549", // AvalancheC
                "0x68208D99746b805a1Ae41421950A47b711E35681", // Base
            ],
            Self::Mayachain => &[
                "0xe3985E6b61b814F7Cdb188766562ba71b446B46d", // Ethereum
                "0x700E97ef07219440487840Dc472E7120A7FF11F4", // Arbitrum
            ],
        }
    }
}
