use primitives::Chain;

pub const BASE_URL: &str = "https://web3.okx.com";

pub const EVM_NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const SOLANA_NATIVE_TOKEN_ADDRESS: &str = "11111111111111111111111111111111";
pub const TRON_NATIVE_TOKEN_ADDRESS: &str = "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb";
pub const TRON_DEX_TOKEN_APPROVE_ADDRESS: &str = "THRAE2VhGNAcvPKtT96AqyXtSQwhiU1XL8";

const DEFAULT_EVM_GAS_LIMIT: u64 = 920_000;

const SOLANA_DEX_IDS: &str = "277,278,279,343,72,103,284,338,372,403,444,483,357,345,459,457,475,342";
const TRON_DEX_IDS: &str = "64,98,596"; // Sunswap

pub fn chain_index(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some("501"),
        Chain::Tron => Some("195"),
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::Fantom
        | Chain::Manta
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Plasma
        | Chain::Hyperliquid
        | Chain::Sonic
        | Chain::Unichain
        | Chain::Monad
        | Chain::XLayer
        | Chain::Robinhood => Some(chain.config().network_id),
        _ => None,
    }
}

pub fn dex_ids(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some(SOLANA_DEX_IDS),
        Chain::Tron => Some(TRON_DEX_IDS),
        _ => None,
    }
}

pub fn evm_gas_limit(chain: Chain) -> u64 {
    match chain {
        Chain::Manta => 600_000,
        Chain::ZkSync => 2_000_000,
        Chain::Mantle => 2_000_000_000,
        _ => DEFAULT_EVM_GAS_LIMIT,
    }
}
