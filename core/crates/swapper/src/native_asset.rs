use primitives::{AssetId, EVMChain};

pub fn requires_native_wrapping(asset_id: &AssetId) -> bool {
    asset_id.is_native() && EVMChain::from_chain(asset_id.chain).and_then(|chain| chain.native_asset_contract()).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_native_wrapping() {
        use primitives::Chain;

        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Tempo)));
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Celo)));
        assert!(requires_native_wrapping(&AssetId::from_chain(Chain::Ethereum)));
    }
}
