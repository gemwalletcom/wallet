use std::{collections::HashSet, error::Error};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetBasic, AssetProperties, AssetScore, Chain, asset_id::AssetId, asset_type::AssetType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: AssetId,
    #[typeshare(skip)]
    pub chain: Chain,
    #[typeshare(skip)]
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ChainAsset {
    pub asset: Asset,
    pub network_name: String,
}

impl ChainAsset {
    pub fn from_chain(chain: Chain) -> Self {
        match chain {
            Chain::Ethereum => ChainAsset::new(chain, "Ethereum", "ETH", 18),
            Chain::Bitcoin => ChainAsset::new(chain, "Bitcoin", "BTC", 8),
            Chain::BitcoinCash => ChainAsset::new(chain, "Bitcoin Cash", "BCH", 8),
            Chain::Litecoin => ChainAsset::new(chain, "Litecoin", "LTC", 8),
            Chain::SmartChain => ChainAsset::new(chain, "BNB Chain", "BNB", 18),
            Chain::Polygon => ChainAsset::new(chain, "Polygon", "POL", 18),
            Chain::AvalancheC => ChainAsset::new(chain, "Avalanche", "AVAX", 18),
            Chain::Solana => ChainAsset::new(chain, "Solana", "SOL", 9),
            Chain::Thorchain => ChainAsset::new(chain, "Thorchain", "RUNE", 8),
            Chain::Mayachain => ChainAsset::new(chain, "Maya", "CACAO", 10),
            Chain::Cosmos => ChainAsset::new(chain, "Cosmos", "ATOM", 6),
            Chain::Osmosis => ChainAsset::new(chain, "Osmosis", "OSMO", 6),
            Chain::Celestia => ChainAsset::new(chain, "Celestia", "TIA", 6),
            Chain::Arbitrum => ChainAsset::with_network_name(chain, "Arbitrum", "Arbitrum ETH", "ETH", 18),
            Chain::Ton => ChainAsset::with_network_name(chain, "TON", "Gram", "GRAM", 9),
            Chain::Tron => ChainAsset::new(chain, "TRON", "TRX", 6),
            Chain::Doge => ChainAsset::new(chain, "Dogecoin", "DOGE", 8),
            Chain::Zcash => ChainAsset::new(chain, "Zcash", "ZEC", 8),
            Chain::Optimism => ChainAsset::with_network_name(chain, "Optimism", "Optimism ETH", "ETH", 18),
            Chain::Aptos => ChainAsset::new(chain, "Aptos", "APT", 8),
            Chain::Base => ChainAsset::with_network_name(chain, "Base", "Base ETH", "ETH", 18),
            Chain::Sui => ChainAsset::new(chain, "Sui", "SUI", 9),
            Chain::Xrp => ChainAsset::new(chain, "XRP", "XRP", 6),
            Chain::OpBNB => ChainAsset::new(chain, "opBNB", "BNB", 18),
            Chain::Fantom => ChainAsset::new(chain, "Fantom", "FTM", 18),
            Chain::Gnosis => ChainAsset::new(chain, "Gnosis Chain", "xDai", 18),
            Chain::Injective => ChainAsset::new(chain, "Injective", "INJ", 18),
            Chain::Sei => ChainAsset::new(chain, "Sei", "SEI", 6),
            Chain::SeiEvm => ChainAsset::new(chain, "Sei EVM", "SEI", 18),
            Chain::Manta => ChainAsset::with_network_name(chain, "Manta", "Manta ETH", "ETH", 18),
            Chain::Blast => ChainAsset::with_network_name(chain, "Blast", "Blast ETH", "ETH", 18),
            Chain::Noble => ChainAsset::new(chain, "Noble", "USDC", 6),
            Chain::ZkSync => ChainAsset::with_network_name(chain, "zkSync", "zkSync ETH", "ETH", 18),
            Chain::Linea => ChainAsset::with_network_name(chain, "Linea", "Linea ETH", "ETH", 18),
            Chain::Mantle => ChainAsset::new(chain, "Mantle", "MNT", 18),
            Chain::Celo => ChainAsset::new(chain, "Celo", "CELO", 18),
            Chain::Near => ChainAsset::new(chain, "Near", "NEAR", 24),
            Chain::World => ChainAsset::with_network_name(chain, "World", "World ETH", "ETH", 18),
            Chain::Stellar => ChainAsset::new(chain, "Stellar", "XLM", 7),
            Chain::Sonic => ChainAsset::new(chain, "Sonic", "S", 18),
            Chain::Algorand => ChainAsset::new(chain, "Algorand", "ALGO", 6),
            Chain::Polkadot => ChainAsset::new(chain, "Polkadot", "DOT", 10),
            Chain::Plasma => ChainAsset::new(chain, "Plasma", "XPL", 18),
            Chain::Cardano => ChainAsset::new(chain, "Cardano", "ADA", 6),
            Chain::Abstract => ChainAsset::new(chain, "Abstract", "ETH", 18),
            Chain::Berachain => ChainAsset::new(chain, "Berachain", "BERA", 18),
            Chain::Ink => ChainAsset::with_network_name(chain, "Ink", "Ink ETH", "ETH", 18),
            Chain::Unichain => ChainAsset::with_network_name(chain, "Unichain", "Unichain ETH", "ETH", 18),
            Chain::Hyperliquid => ChainAsset::new(chain, "HyperEVM", "HYPE", 18),
            Chain::HyperCore => ChainAsset::new(chain, "Hyperliquid", "HYPE", 8),
            Chain::Monad => ChainAsset::new(chain, "Monad", "MON", 18),
            Chain::XLayer => ChainAsset::new(chain, "X Layer", "OKB", 18),
            Chain::Robinhood => ChainAsset::with_network_name(chain, "Robinhood", "Robinhood ETH", "ETH", 18),
            Chain::Stable => ChainAsset::new(chain, "Stable", "USDT0", 18),
            Chain::Tempo => ChainAsset::new(chain, "Tempo", "USD", 6),
        }
    }

    fn new(chain: Chain, name: &str, symbol: &str, decimals: i32) -> Self {
        Self::with_network_name(chain, name, name, symbol, decimals)
    }

    fn with_network_name(chain: Chain, network_name: &str, name: &str, symbol: &str, decimals: i32) -> Self {
        Self {
            asset: chain.new_asset(name, symbol, decimals, AssetType::NATIVE),
            network_name: network_name.to_string(),
        }
    }
}

impl Chain {
    pub fn new_asset(&self, name: impl Into<String>, symbol: impl Into<String>, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: self.as_asset_id(),
            chain: *self,
            token_id: None,
            name: name.into(),
            symbol: symbol.into(),
            decimals,
            asset_type,
        }
    }
}

impl Asset {
    pub fn new(id: AssetId, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: id.clone(),
            chain: id.chain,
            token_id: id.token_id.clone(),
            name,
            symbol,
            decimals,
            asset_type,
        }
    }

    pub fn chain(&self) -> Chain {
        self.id.chain
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }

    pub fn as_basic_primitive(&self) -> AssetBasic {
        AssetBasic::new(self.clone(), AssetProperties::default(self.id.clone()), self.default_score())
    }

    pub fn default_score(&self) -> AssetScore {
        AssetScore::new(self.id.default_rank())
    }

    pub fn from_chain(chain: Chain) -> Asset {
        ChainAsset::from_chain(chain).asset
    }
}

pub trait AssetVecExt {
    fn ids(&self) -> Vec<AssetId>;
    fn ids_set(&self) -> HashSet<AssetId>;
    fn asset(&self, asset_id: AssetId) -> Option<Asset>;
    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>>;
}

impl AssetVecExt for Vec<Asset> {
    fn ids(&self) -> Vec<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn ids_set(&self) -> HashSet<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn asset(&self, asset_id: AssetId) -> Option<Asset> {
        self.iter().find(|x| x.id == asset_id).cloned()
    }

    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>> {
        self.iter().find(|x| x.id == asset_id).ok_or("Asset not found".into())
    }
}

#[cfg(test)]
mod chain_asset_tests {
    use super::*;

    #[test]
    fn chain_asset_separates_network_and_native_asset_names() {
        let ton = ChainAsset::from_chain(Chain::Ton);
        assert_eq!(ton.network_name, "TON");
        assert_eq!(ton.asset.name, "Gram");
        assert_eq!(ton.asset.symbol, "GRAM");

        let base = ChainAsset::from_chain(Chain::Base);
        assert_eq!(base.network_name, "Base");
        assert_eq!(base.asset.name, "Base ETH");
        assert_eq!(base.asset.symbol, "ETH");
    }

    #[test]
    fn asset_from_chain_preserves_existing_native_asset_accessor() {
        assert_eq!(Asset::from_chain(Chain::Ton), ChainAsset::from_chain(Chain::Ton).asset);
    }
}

pub trait AssetHashSetExt {
    fn ids(&self) -> Vec<String>;
}

impl AssetHashSetExt for HashSet<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().map(|x| x.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let asset = Asset::from_chain(Chain::Gnosis);

        assert_eq!(asset.symbol, "xDai");
    }

    #[test]
    fn test_sei_evm_asset() {
        let asset = Asset::from_chain(Chain::SeiEvm);

        assert_eq!(asset.name, "Sei EVM");
        assert_eq!(asset.symbol, "SEI");
        assert_eq!(asset.decimals, 18);
    }

    #[test]
    fn test_as_basic_primitive_score() {
        let native = Asset::from_chain(Chain::Robinhood).as_basic_primitive();
        let token = Asset::new(AssetId::from_token(Chain::Robinhood, "0x123"), "Token".to_string(), "TKN".to_string(), 18, AssetType::ERC20).as_basic_primitive();

        let known_token = crate::known_assets::SOLANA_USDC.as_basic_primitive();

        assert_eq!(native.score.rank, Chain::Robinhood.rank());
        assert_eq!(token.score.rank, AssetScore::default().rank);
        assert!(known_token.score.rank > crate::asset_score::AssetRank::Trivial.threshold());
    }
}
