use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::chain_config::{ChainConfig, get_chain_config};
use crate::{AssetId, AssetType, ChainType};

#[derive(Copy, Clone, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    BitcoinCash,
    Litecoin,
    Ethereum,
    SmartChain,
    Solana,
    Polygon,
    Thorchain,
    Mayachain,
    Cosmos,
    Osmosis,
    Arbitrum,
    Ton,
    Tron,
    Doge,
    Zcash,
    Optimism,
    Aptos,
    Base,
    AvalancheC,
    Sui,
    Xrp,
    OpBNB,
    Fantom,
    Gnosis,
    Celestia,
    Injective,
    Sei,
    SeiEvm,
    Manta,
    Blast,
    Noble,
    ZkSync,
    Linea,
    Mantle,
    Celo,
    Near,
    World,
    Stellar,
    Sonic,
    Algorand,
    Polkadot,
    Plasma,
    Cardano,
    Abstract,
    Berachain,
    Ink,
    Unichain,
    Hyperliquid, // HyperEVM
    HyperCore,   // HyperCore native chain
    Monad,
    XLayer,
    Robinhood,
    Stable,
}

impl fmt::Debug for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl Chain {
    pub fn config(&self) -> &'static ChainConfig {
        get_chain_config(*self)
    }

    pub fn as_denom(&self) -> Option<&str> {
        self.config().denom.as_deref()
    }

    pub fn as_asset_id(&self) -> AssetId {
        AssetId::from_chain(*self)
    }

    pub fn network_id(&self) -> &str {
        self.config().network_id
    }

    pub fn network_id_value(&self) -> Option<u64> {
        let network_id = self.network_id();
        network_id
            .parse()
            .ok()
            .or_else(|| network_id.strip_prefix("0x").and_then(|hex| u64::from_str_radix(hex, 16).ok()))
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Self::iter().find(|&chain| chain.network_id_value().is_some_and(|network_id| network_id == chain_id))
    }

    pub fn is_utxo(&self) -> bool {
        self.config().is_utxo
    }

    pub fn as_slip44(&self) -> i64 {
        self.config().slip44
    }

    pub fn chain_type(&self) -> ChainType {
        self.config().chain_type.clone()
    }

    pub fn default_asset_type(&self) -> Option<AssetType> {
        self.config().default_asset_type.clone()
    }

    pub fn account_activation_fee(&self) -> Option<i32> {
        self.config().account_activation_fee
    }

    pub fn token_activation_fee(&self) -> Option<i32> {
        self.config().token_activation_fee
    }

    pub fn minimum_account_balance(&self) -> Option<u64> {
        self.config().minimum_account_balance
    }

    pub fn is_swap_supported(&self) -> bool {
        self.config().is_swap_supported
    }

    pub fn is_stake_supported(&self) -> bool {
        self.config().stake.is_some()
    }

    pub fn is_nft_supported(&self) -> bool {
        self.config().is_nft_supported
    }

    pub fn is_defi_supported(&self) -> bool {
        self.config().is_defi_supported
    }

    // milliseconds
    pub fn block_time(&self) -> u32 {
        self.config().block_time
    }

    pub fn rank(&self) -> i32 {
        self.config().rank
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn stakeable() -> Vec<Self> {
        Self::all().into_iter().filter(|x| x.is_stake_supported()).collect()
    }

    pub fn perpetual_chains() -> Vec<Self> {
        vec![Self::HyperCore]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mayachain_swap_not_supported() {
        assert!(!Chain::Mayachain.is_swap_supported());
    }

    #[test]
    fn test_robinhood_swap_supported() {
        assert!(Chain::Robinhood.is_swap_supported());
    }

    #[test]
    fn test_defi_supported() {
        assert!(Chain::Ethereum.is_defi_supported());
        assert!(Chain::Base.is_defi_supported());
        assert!(Chain::Solana.is_defi_supported());
        assert!(!Chain::Bitcoin.is_defi_supported());
    }

    #[test]
    fn test_from_chain_id_supports_hex_network_id() {
        assert_eq!(Chain::from_chain_id(728_126_428), Some(Chain::Tron));
    }
}
