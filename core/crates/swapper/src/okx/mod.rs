mod auth;
mod client;
mod constants;
mod model;
mod provider;
mod referral;
#[cfg(test)]
mod testkit;

use crate::SwapperChainAsset;
use primitives::Chain;

pub use model::OkxClientConfig;
pub use provider::OkxProvider;

pub(crate) fn support_assets() -> Vec<SwapperChainAsset> {
    vec![
        SwapperChainAsset::All(Chain::Solana),
        SwapperChainAsset::All(Chain::Tron),
        SwapperChainAsset::All(Chain::Ethereum),
        SwapperChainAsset::All(Chain::SmartChain),
        SwapperChainAsset::All(Chain::Polygon),
        SwapperChainAsset::All(Chain::Arbitrum),
        SwapperChainAsset::All(Chain::Optimism),
        SwapperChainAsset::All(Chain::Base),
        SwapperChainAsset::All(Chain::AvalancheC),
        SwapperChainAsset::All(Chain::Fantom),
        SwapperChainAsset::All(Chain::Manta),
        SwapperChainAsset::All(Chain::Blast),
        SwapperChainAsset::All(Chain::ZkSync),
        SwapperChainAsset::All(Chain::Linea),
        SwapperChainAsset::All(Chain::Mantle),
        SwapperChainAsset::All(Chain::Hyperliquid),
        SwapperChainAsset::All(Chain::Sonic),
        SwapperChainAsset::All(Chain::Unichain),
        SwapperChainAsset::All(Chain::Monad),
        SwapperChainAsset::All(Chain::XLayer),
    ]
}
