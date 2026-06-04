use super::THORChainNetwork;
use gem_evm::address::ethereum_address_checksum;
use primitives::{Asset, Chain, known_assets::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainName {
    chain: Chain,
    long_name: &'static str,
    short_name: &'static str,
    token_assets: ChainTokenAssets,
}

impl ChainName {
    const fn new(chain: Chain, long_name: &'static str, short_name: &'static str, token_assets: ChainTokenAssets) -> Self {
        Self {
            chain,
            long_name,
            short_name,
            token_assets,
        }
    }

    pub fn supported(network: THORChainNetwork) -> &'static [ChainName] {
        match network {
            THORChainNetwork::Thorchain => THORCHAIN_NAMES,
            THORChainNetwork::Mayachain => MAYACHAIN_NAMES,
        }
    }

    pub fn long_name(&self) -> &str {
        self.long_name
    }

    pub fn chain(&self) -> Chain {
        self.chain
    }

    pub fn from_chain(network: THORChainNetwork, chain: Chain) -> Option<ChainName> {
        Self::supported(network).iter().find(|name| name.chain == chain).copied()
    }

    pub fn from_symbol(network: THORChainNetwork, symbol: &str) -> Option<ChainName> {
        Self::supported(network).iter().find(|name| name.long_name == symbol || name.short_name == symbol).copied()
    }

    pub fn short_name(&self) -> &str {
        self.short_name
    }

    pub fn token_asset(&self, token_id: &str) -> Option<Asset> {
        self.token_assets().into_iter().find(|asset| asset.id.token_id.as_ref().is_some_and(|id| id == token_id))
    }

    pub fn token_assets(&self) -> Vec<Asset> {
        match self.token_assets {
            ChainTokenAssets::None => vec![],
            ChainTokenAssets::Thorchain => vec![(*THORCHAIN_TCY).clone()],
            ChainTokenAssets::ThorchainEthereum => vec![(*ETHEREUM_USDT).clone(), (*ETHEREUM_USDC).clone(), (*ETHEREUM_DAI).clone(), (*ETHEREUM_WBTC).clone()],
            ChainTokenAssets::MayachainEthereum => vec![(*ETHEREUM_USDT).clone(), (*ETHEREUM_USDC).clone()],
            ChainTokenAssets::SmartChain => vec![(*SMARTCHAIN_USDT).clone(), (*SMARTCHAIN_USDC).clone()],
            ChainTokenAssets::Avalanche => vec![(*AVALANCHE_USDT).clone(), (*AVALANCHE_USDC).clone()],
            ChainTokenAssets::Base => vec![(*BASE_USDC).clone(), (*BASE_CBBTC).clone()],
            ChainTokenAssets::Tron => vec![(*TRON_USDT).clone()],
            ChainTokenAssets::Arbitrum => vec![(*ARBITRUM_USDC).clone(), (*ARBITRUM_USDT).clone()],
        }
    }

    pub fn is_evm_chain(&self) -> bool {
        matches!(self.chain, Chain::Ethereum | Chain::SmartChain | Chain::AvalancheC | Chain::Base | Chain::Arbitrum)
    }

    pub fn checksum_address(&self, address: &str) -> String {
        if self.is_evm_chain() {
            let address = address.strip_prefix("0X").unwrap_or(address);
            ethereum_address_checksum(address).unwrap_or(address.to_string())
        } else {
            address.to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChainTokenAssets {
    None,
    Thorchain,
    ThorchainEthereum,
    MayachainEthereum,
    SmartChain,
    Avalanche,
    Base,
    Tron,
    Arbitrum,
}

const THORCHAIN_NAMES: &[ChainName] = &[
    ChainName::new(Chain::Doge, "DOGE", "d", ChainTokenAssets::None),
    ChainName::new(Chain::Thorchain, "THOR", "r", ChainTokenAssets::Thorchain),
    ChainName::new(Chain::Ethereum, "ETH", "e", ChainTokenAssets::ThorchainEthereum),
    ChainName::new(Chain::Cosmos, "GAIA", "g", ChainTokenAssets::None),
    ChainName::new(Chain::Bitcoin, "BTC", "b", ChainTokenAssets::None),
    ChainName::new(Chain::BitcoinCash, "BCH", "c", ChainTokenAssets::None),
    ChainName::new(Chain::Litecoin, "LTC", "l", ChainTokenAssets::None),
    ChainName::new(Chain::SmartChain, "BSC", "s", ChainTokenAssets::SmartChain),
    ChainName::new(Chain::AvalancheC, "AVAX", "a", ChainTokenAssets::Avalanche),
    ChainName::new(Chain::Base, "BASE", "f", ChainTokenAssets::Base),
    ChainName::new(Chain::Xrp, "XRP", "x", ChainTokenAssets::None),
    ChainName::new(Chain::Tron, "TRON", "tr", ChainTokenAssets::Tron),
    ChainName::new(Chain::Solana, "SOL", "o", ChainTokenAssets::None),
    ChainName::new(Chain::Zcash, "ZEC", "z", ChainTokenAssets::None),
];

const MAYACHAIN_NAMES: &[ChainName] = &[
    ChainName::new(Chain::Thorchain, "THOR", "r", ChainTokenAssets::None),
    ChainName::new(Chain::Bitcoin, "BTC", "b", ChainTokenAssets::None),
    ChainName::new(Chain::Ethereum, "ETH", "e", ChainTokenAssets::MayachainEthereum),
    ChainName::new(Chain::Arbitrum, "ARB", "a", ChainTokenAssets::Arbitrum),
    ChainName::new(Chain::Zcash, "ZEC", "z", ChainTokenAssets::None),
    ChainName::new(Chain::Cardano, "ADA", "aa", ChainTokenAssets::None),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thorchain_symbols() {
        for name in ChainName::supported(THORChainNetwork::Thorchain) {
            assert_eq!(ChainName::from_symbol(THORChainNetwork::Thorchain, name.long_name()), Some(*name));
        }

        assert_eq!(ChainName::from_symbol(THORChainNetwork::Thorchain, "a").map(|name| name.chain()), Some(Chain::AvalancheC));
        assert_eq!(ChainName::from_chain(THORChainNetwork::Thorchain, Chain::Arbitrum), None);
        assert_eq!(ChainName::from_symbol(THORChainNetwork::Thorchain, "ARB"), None);
    }

    #[test]
    fn test_mayachain_symbols() {
        for name in ChainName::supported(THORChainNetwork::Mayachain) {
            assert_eq!(ChainName::from_symbol(THORChainNetwork::Mayachain, name.long_name()), Some(*name));
        }

        assert_eq!(ChainName::from_symbol(THORChainNetwork::Mayachain, "a").map(|name| name.chain()), Some(Chain::Arbitrum));
        assert_eq!(ChainName::from_symbol(THORChainNetwork::Mayachain, "aa").map(|name| name.chain()), Some(Chain::Cardano));
        assert_eq!(ChainName::from_symbol(THORChainNetwork::Mayachain, "r").map(|name| name.chain()), Some(Chain::Thorchain));
        assert_eq!(ChainName::from_chain(THORChainNetwork::Mayachain, Chain::Arbitrum).unwrap().long_name(), "ARB");
    }

    #[test]
    fn test_checksum_address_preserves_non_evm_case() {
        let zcash = "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML";
        let name = ChainName::from_chain(THORChainNetwork::Mayachain, Chain::Zcash).unwrap();
        assert_eq!(name.checksum_address(zcash), zcash);
    }
}
