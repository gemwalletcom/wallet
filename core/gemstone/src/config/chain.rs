use primitives::{Asset, AssetType, BitcoinChain, Chain, ChainType, EVMChain};

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct ChainConfig {
    pub default_asset_type: Option<AssetType>,
    pub icon_chain: Chain,
    pub is_memo_supported: bool,
}

pub fn get_chain_config(chain: Chain) -> ChainConfig {
    ChainConfig {
        default_asset_type: chain.default_asset_type(),
        icon_chain: icon_chain(chain),
        is_memo_supported: is_memo_supported(chain),
    }
}

pub fn icon_chain(chain: Chain) -> Chain {
    match chain {
        Chain::SeiEvm => Chain::Sei,
        chain => chain,
    }
}

pub(crate) fn badge_chain(chain: Chain) -> Option<Chain> {
    is_ether_layer2(chain).then_some(chain)
}

pub(crate) fn is_ether_layer2(chain: Chain) -> bool {
    EVMChain::from_chain(chain).is_some_and(|chain| chain.is_ethereum_layer2()) && Asset::from_chain(chain).symbol == Asset::from_chain(Chain::Ethereum).symbol
}

pub fn supports_nft_transfer(chain: Chain) -> bool {
    chain.is_nft_supported() && matches!(chain.chain_type(), ChainType::Ethereum | ChainType::Ton | ChainType::Solana)
}

pub fn is_memo_supported(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Solana | ChainType::Cosmos | ChainType::Ton | ChainType::Xrp | ChainType::Stellar | ChainType::Algorand => true,
        ChainType::Ethereum | ChainType::Bitcoin | ChainType::Near | ChainType::Tron | ChainType::Aptos | ChainType::Sui | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => false,
    }
}

pub fn account_activation_fee_url(chain: Chain) -> Option<String> {
    match chain {
        Chain::Xrp => Some("https://xrpl.org/docs/concepts/accounts/reserves#base-reserve-and-owner-reserve".into()),
        Chain::Stellar => Some("https://developers.stellar.org/docs/learn/fundamentals/lumens#minimum-balance".into()),
        Chain::Algorand => Some("https://developer.algorand.org/docs/features/accounts/#minimum-balance".into()),
        _ => None,
    }
}

#[allow(clippy::match_like_matches_macro)]
pub fn custom_fee_enabled(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Bitcoin => true,
        _ => false,
    }
}

pub fn minimum_custom_fee_rate(chain: Chain) -> Option<u32> {
    match chain.chain_type() {
        ChainType::Bitcoin => BitcoinChain::from_chain(chain).map(|chain| chain.minimum_custom_fee_rate()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_transfer_needs_a_supported_chain_type() {
        assert!(supports_nft_transfer(Chain::Ethereum));
        assert!(supports_nft_transfer(Chain::Solana));
        assert!(supports_nft_transfer(Chain::Ton));
        assert!(!supports_nft_transfer(Chain::Bitcoin));
        assert!(!supports_nft_transfer(Chain::Tron));
    }

    #[test]
    fn test_chain_icon_is_its_own_logo_and_only_ethereum_layer2_chains_badge() {
        assert_eq!(icon_chain(Chain::Base), Chain::Base);
        assert_eq!(icon_chain(Chain::SeiEvm), Chain::Sei);
        assert_eq!(icon_chain(Chain::OpBNB), Chain::OpBNB);
        assert_eq!(icon_chain(Chain::Ethereum), Chain::Ethereum);

        assert_eq!(badge_chain(Chain::Base), Some(Chain::Base));
        assert_eq!(badge_chain(Chain::Celo), None);
        assert_eq!(badge_chain(Chain::Mantle), None);
        assert_eq!(badge_chain(Chain::XLayer), None);
        assert_eq!(badge_chain(Chain::OpBNB), None);
        assert_eq!(badge_chain(Chain::Bitcoin), None);
        assert_eq!(badge_chain(Chain::Ethereum), None);
    }
}
