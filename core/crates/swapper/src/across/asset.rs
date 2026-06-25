use crate::{SwapperError, eth_address};
use alloy_primitives::Address;
use gem_evm::across::deployment::AcrossDeployment;
use gem_tron::address::TronAddress;
use primitives::{AssetId, Chain};

pub(in crate::across) fn across_asset_id(asset: &AssetId) -> Option<AssetId> {
    match asset.chain {
        Chain::Tron => Some(asset.clone()),
        _ => eth_address::convert_native_to_weth(asset),
    }
}

pub(in crate::across) fn parse_address(chain: Chain, address: &str) -> Result<Address, SwapperError> {
    match chain {
        Chain::Tron => {
            let address = TronAddress::parse_hex_or_base58(address)?;
            Ok(Address::from_slice(address.account_id()))
        }
        _ => eth_address::parse_str(address),
    }
}

pub(in crate::across) fn supported_asset_for_token(chain: Chain, token_address: &str) -> Option<AssetId> {
    let token_address = parse_address(chain, token_address).ok()?;
    AcrossDeployment::supported_assets().get(&chain)?.iter().find_map(|asset| {
        let across_asset = across_asset_id(asset)?;
        let asset_address = parse_address(across_asset.chain, across_asset.token_id.as_deref()?).ok()?;
        if asset_address != token_address {
            return None;
        }

        let native_asset = AssetId::from_chain(asset.chain);
        if across_asset_id(&native_asset).as_ref() == Some(asset) {
            Some(native_asset)
        } else {
            Some(asset.clone())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::*;

    #[test]
    fn test_supported_asset_for_token() {
        assert_eq!(
            supported_asset_for_token(Chain::Ethereum, ETHEREUM_WETH_TOKEN_ID),
            Some(AssetId::from_chain(Chain::Ethereum))
        );
        assert_eq!(
            supported_asset_for_token(Chain::Arbitrum, ARBITRUM_WETH_TOKEN_ID),
            Some(AssetId::from_chain(Chain::Arbitrum))
        );
        assert_eq!(
            supported_asset_for_token(Chain::Ethereum, &ETHEREUM_USDC_TOKEN_ID.to_ascii_lowercase()),
            Some(ETHEREUM_USDC_ASSET_ID.clone())
        );
        assert_eq!(supported_asset_for_token(Chain::Tron, TRON_USDT_TOKEN_ID), Some(TRON_USDT_ASSET_ID.clone()));
        assert_eq!(
            supported_asset_for_token(Chain::Tron, "0xa614f803b6fd780986a42c78ec9c7f77e6ded13c"),
            Some(TRON_USDT_ASSET_ID.clone())
        );
        assert_eq!(supported_asset_for_token(Chain::Bitcoin, "0x123"), None);
    }
}
