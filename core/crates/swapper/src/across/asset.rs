use crate::eth_address;
use alloy_primitives::Address;
use gem_evm::across::deployment::AcrossDeployment;
use primitives::{AssetId, Chain};

pub(in crate::across) fn supported_asset_for_token(chain: Chain, token_address: Address) -> Option<AssetId> {
    if chain != Chain::Tron {
        return AcrossDeployment::supported_asset_for_token(chain, token_address);
    }

    AcrossDeployment::supported_assets().get(&chain)?.iter().find_map(|asset| {
        let asset_address = eth_address::parse(asset.chain, asset.token_id.as_deref()?).ok()?;
        if asset_address != token_address {
            return None;
        }
        Some(asset.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::*;

    #[test]
    fn test_supported_asset_for_token() {
        let supported_asset_for_token_id = |chain: Chain, token_id: &str| supported_asset_for_token(chain, eth_address::parse(chain, token_id).unwrap());

        assert_eq!(supported_asset_for_token_id(Chain::Ethereum, ETHEREUM_WETH_TOKEN_ID), Some(AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(supported_asset_for_token_id(Chain::Arbitrum, ARBITRUM_WETH_TOKEN_ID), Some(AssetId::from_chain(Chain::Arbitrum)));
        assert_eq!(supported_asset_for_token_id(Chain::Ethereum, &ETHEREUM_USDC_TOKEN_ID.to_ascii_lowercase()), Some(ETHEREUM_USDC_ASSET_ID.clone()));
        assert_eq!(supported_asset_for_token_id(Chain::Tron, TRON_USDT_TOKEN_ID), Some(TRON_USDT_ASSET_ID.clone()));
        assert_eq!(supported_asset_for_token_id(Chain::Robinhood, ROBINHOOD_WETH_TOKEN_ID), Some(AssetId::from_chain(Chain::Robinhood)));
        assert_eq!(supported_asset_for_token_id(Chain::Robinhood, ROBINHOOD_USDG_TOKEN_ID), Some(ROBINHOOD_USDG_ASSET_ID.clone()));
        assert_eq!(supported_asset_for_token_id(Chain::Arc, ARC_USDC_TOKEN_ID), Some(AssetId::from_chain(Chain::Arc)));
        assert_eq!(supported_asset_for_token_id(Chain::Blast, BLAST_WETH_TOKEN_ID), Some(AssetId::from_chain(Chain::Blast)));
        assert_eq!(supported_asset_for_token(Chain::Bitcoin, Address::ZERO), None);
    }
}
