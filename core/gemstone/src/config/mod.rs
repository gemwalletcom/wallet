pub mod chain;
pub mod docs;
pub mod fee_config;
pub mod fiat_config;
pub mod node;
pub mod perpetual_config;
pub mod public;
pub mod rewards;
pub mod scan_config;
pub mod search_config;
pub mod social;
pub mod stake;
pub mod swap_config;
pub mod validators;
pub mod wallet_connect;

use crate::config::chain::ChainConfig;
use primitives::{Chain, StakeChain, node_config::NodeRegion};
use std::str::FromStr;

use {
    fee_config::{FeeConfig, get_fee_config},
    fiat_config::{FiatConfig, get_fiat_config},
    perpetual_config::{PerpetualConfig, get_perpetual_config, leverage_options, select_leverage},
    scan_config::{ScanConfig, get_scan_config},
    search_config::{WalletSearchConfig, get_wallet_search_config},
    stake::{StakeChainConfig, get_stake_config},
    swap_config::{SwapConfig, get_swap_config},
    wallet_connect::{WalletConnectConfig, get_wallet_connect_config},
};

/// Config
#[derive(uniffi::Object)]
struct Config {}
#[uniffi::export]
impl Config {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_stake_config(&self, chain: &str) -> StakeChainConfig {
        let chain = StakeChain::from_str(chain).unwrap();
        get_stake_config(chain)
    }

    fn get_swap_config(&self) -> SwapConfig {
        get_swap_config()
    }

    fn get_perpetual_config(&self) -> PerpetualConfig {
        get_perpetual_config()
    }

    fn get_fiat_config(&self) -> FiatConfig {
        get_fiat_config()
    }

    fn get_wallet_search_config(&self) -> WalletSearchConfig {
        get_wallet_search_config()
    }

    fn get_scan_config(&self) -> ScanConfig {
        get_scan_config()
    }

    fn leverage_options(&self, max_leverage: u8) -> Vec<u8> {
        leverage_options(max_leverage)
    }

    fn select_leverage(&self, desired: u8, options: Vec<u8>) -> u8 {
        select_leverage(desired, &options)
    }

    fn get_chain_config(&self, chain: Chain) -> ChainConfig {
        crate::config::chain::get_chain_config(chain)
    }

    fn get_fee_config(&self, chain: Chain) -> FeeConfig {
        get_fee_config(chain)
    }

    fn get_wallet_connect_config(&self) -> WalletConnectConfig {
        get_wallet_connect_config()
    }

    fn get_node_regions(&self) -> Vec<NodeRegion> {
        NodeRegion::all()
    }

    fn get_node_url(&self, chain: Chain, region: NodeRegion) -> String {
        region.url(chain)
    }

    fn get_node_region(&self, url: &str) -> Option<NodeRegion> {
        NodeRegion::from_url(url)
    }

    fn get_node_region_flag(&self, region: NodeRegion) -> String {
        region.flag().to_string()
    }

    fn get_node_region_priority(&self, region: NodeRegion) -> i32 {
        region.priority()
    }
}
