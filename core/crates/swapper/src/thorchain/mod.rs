mod asset;
mod chain;
pub mod client;
mod constants;
pub mod memo;
pub mod model;
mod provider;
mod quote_data_mapper;
mod swap_mapper;

use primitives::Chain;
use std::sync::Arc;

use crate::alien::RpcProvider;
use asset::value_to;
use chain::ChainName;
use gem_client::Client;
use strum::Display;

use super::{ProviderType, SwapperError, SwapperProvider};

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const DUST_THRESHOLD_MULTIPLIER: i64 = 2;
const OUTBOUND_DELAY_SECONDS: u32 = 60;

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

    pub fn supported_chains(&self) -> Vec<Chain> {
        ChainName::supported(*self).iter().map(|name| name.chain()).collect()
    }
}

#[derive(Debug)]
pub struct ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub provider: ProviderType,
    pub rpc_provider: Arc<dyn RpcProvider>,
    pub(crate) network: THORChainNetwork,
    pub(crate) client: client::ThorChainSwapClient<C>,
}

impl<C> ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub fn with_client(swap_client: client::ThorChainSwapClient<C>, rpc_provider: Arc<dyn RpcProvider>, network: THORChainNetwork) -> Self {
        Self {
            provider: ProviderType::new(network.provider()),
            rpc_provider,
            network,
            client: swap_client,
        }
    }

    fn get_eta_in_seconds(&self, destination_chain: Chain, total_swap_seconds: Option<u32>) -> u32 {
        destination_chain.block_time() / 1000 + OUTBOUND_DELAY_SECONDS + total_swap_seconds.unwrap_or(0)
    }

    fn map_quote_error(&self, error: SwapperError, decimals: i32) -> SwapperError {
        match error {
            SwapperError::InputAmountError { min_amount: Some(min) } => SwapperError::InputAmountError {
                min_amount: Some(value_to(&min, decimals).to_string()),
            },
            other => other,
        }
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::alien::reqwest_provider::NativeProvider;
    use std::sync::Arc;

    #[test]
    fn test_get_eta_in_seconds() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        assert_eq!(thorchain.get_eta_in_seconds(Chain::Bitcoin, None), 660);
        assert_eq!(thorchain.get_eta_in_seconds(Chain::Bitcoin, Some(1200)), 1860);
        assert_eq!(thorchain.get_eta_in_seconds(Chain::SmartChain, Some(648)), 709);
    }

    #[test]
    fn test_map_quote_error() {
        let thorchain = ThorChain::new(Arc::new(NativeProvider::default()));

        let cases = vec![(18, "6614750000000000"), (8, "661475"), (6, "6614")];

        for (decimals, expected) in cases {
            let error = SwapperError::InputAmountError {
                min_amount: Some("661475".to_string()),
            };
            let result = thorchain.map_quote_error(error, decimals);
            assert_eq!(
                result,
                SwapperError::InputAmountError {
                    min_amount: Some(expected.to_string())
                }
            );
        }

        let error = SwapperError::InputAmountError { min_amount: None };
        assert_eq!(thorchain.map_quote_error(error, 18), SwapperError::InputAmountError { min_amount: None });

        let error = SwapperError::NotSupportedAsset;
        assert_eq!(thorchain.map_quote_error(error, 18), SwapperError::NotSupportedAsset);
    }
}
