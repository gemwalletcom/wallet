#[cfg(test)]
use primitives::Chain;
use primitives::{AssetId, EVMChain};

pub fn requires_native_wrapping(asset_id: &AssetId) -> bool {
    asset_id.is_native() && EVMChain::from_chain(asset_id.chain).is_some_and(|chain| chain.native_asset_contract().is_none() && chain.weth_contract().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_native_wrapping() {
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Celo)));
        assert!(requires_native_wrapping(&AssetId::from_chain(Chain::Ethereum)));
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Tempo)));
    }
}
