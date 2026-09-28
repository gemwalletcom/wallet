pub mod chain;
pub mod docs;
pub mod fee_config;
pub mod fiat_config;
pub mod image;
pub mod node;
pub mod perpetual_config;
pub mod public;
pub mod rewards;
pub mod search_config;
pub mod social;
pub mod stake;
pub mod swap_config;
pub mod validators;
pub mod wallet_connect;

use primitives::Platform;

pub fn with_utm_source(url: &str, platform: Platform) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}utm_source=gemwallet_{}", platform.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::public::PublicUrl;

    #[test]
    fn test_a_url_with_a_query_gains_the_source_as_another_parameter() {
        assert_eq!(PublicUrl::PlayStore.url_for(Platform::Android), format!("{}&utm_source=gemwallet_android", PublicUrl::PlayStore.url()));
        assert_eq!(PublicUrl::Website.url_for(Platform::IOS), "https://gemwallet.com?utm_source=gemwallet_ios");
    }
}

use crate::config::chain::ChainConfig;
use primitives::{Chain, node_config::NodeRegion};

use {
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

    fn get_swap_config(&self) -> SwapConfig {
        get_swap_config()
    }

    fn get_chain_config(&self, chain: Chain) -> ChainConfig {
        crate::config::chain::get_chain_config(chain)
    }

    fn get_wallet_connect_config(&self) -> WalletConnectConfig {
        get_wallet_connect_config()
    }

    fn get_node_url(&self, chain: Chain, region: NodeRegion) -> String {
        region.url(chain)
    }
}
