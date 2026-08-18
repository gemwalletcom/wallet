use primitives::{AssetId, Chain, EVMChain};

pub fn requires_native_wrapping(asset_id: &AssetId) -> bool {
    asset_id.is_native() && !is_native_erc20(asset_id.chain)
}

pub fn is_native_erc20(chain: Chain) -> bool {
    chain == Chain::Celo || chain == Chain::Tempo
}

pub fn native_erc20_address(chain: &EVMChain) -> Option<&str> {
    if is_native_erc20(chain.to_chain()) { chain.weth_contract() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::TEMPO_PATHUSD_TOKEN_ID;

    #[test]
    fn test_is_native_erc20() {
        assert!(is_native_erc20(Chain::Celo));
        assert!(is_native_erc20(Chain::Tempo));
        assert!(!is_native_erc20(Chain::Ethereum));
    }

    #[test]
    fn test_native_erc20_address() {
        assert_eq!(native_erc20_address(&EVMChain::Tempo), Some(TEMPO_PATHUSD_TOKEN_ID));
        assert_eq!(native_erc20_address(&EVMChain::Celo), EVMChain::Celo.weth_contract());
        assert_eq!(native_erc20_address(&EVMChain::Ethereum), None);
    }

    #[test]
    fn test_requires_native_wrapping() {
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Tempo)));
        assert!(!requires_native_wrapping(&AssetId::from_chain(Chain::Celo)));
        assert!(requires_native_wrapping(&AssetId::from_chain(Chain::Ethereum)));
    }
}
