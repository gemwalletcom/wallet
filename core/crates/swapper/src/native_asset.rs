#[cfg(test)]
use primitives::Chain;
use primitives::{AssetId, EVMChain};

pub fn requires_native_wrapping(asset_id: &AssetId) -> bool {
    asset_id.is_native() && EVMChain::from_chain(asset_id.chain).is_some_and(|chain| chain.native_asset_contract().is_none() && chain.weth_contract().is_some())
}

/// Uniswap v4 settles the native currency directly, so it does not need a wrapped token: any chain with a
/// real native asset and no ERC-20 stand-in for it qualifies, including chains without WETH such as Arc.
pub fn uses_native_currency(asset_id: &AssetId) -> bool {
    asset_id.is_native() && asset_id.chain.has_native_asset() && EVMChain::from_chain(asset_id.chain).is_some_and(|chain| chain.native_asset_contract().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_native_wrapping() {
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Celo)));
        assert!(requires_native_wrapping(&AssetId::from_chain(Chain::Ethereum)));
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Tempo)));
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Arc)));
    }

    #[test]
    fn test_uses_native_currency() {
        assert!(uses_native_currency(&AssetId::from_chain(Chain::Ethereum)));
        assert!(uses_native_currency(&AssetId::from_chain(Chain::Arc)));
        assert!(!uses_native_currency(&AssetId::from_chain(Chain::Celo)));
        assert!(!uses_native_currency(&AssetId::from_chain(Chain::Tempo)));
        assert!(!uses_native_currency(&AssetId::from(
            Chain::Arc,
            Some(primitives::asset_constants::ARC_USDC_TOKEN_ID.to_string())
        )));
    }
}
