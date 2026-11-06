use chain_primitives::format_token_id;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{
    Asset, AssetId, Chain, CosmosDenom, FiatAssetSymbol, FiatProviderName, RequestError, WalletType,
    asset_constants::{HYPERCORE_SPOT_HYPE_ASSET_ID, HYPERCORE_SPOT_UBTC_ASSET_ID, HYPERCORE_SPOT_USDC_ASSET_ID, WORLD_WETH_TOKEN_ID},
    contract_constants::{EVM_ZERO_ADDRESS, SOLANA_SYSTEM_PROGRAM_ID},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FiatMapping {
    pub asset: Asset,
    pub asset_symbol: FiatAssetSymbol,
    pub country_code: String,
    pub unsupported_countries: HashMap<String, Vec<String>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
}

#[derive(Debug, Clone)]
pub struct FiatProviderAsset {
    pub id: String,
    pub provider: FiatProviderName,
    pub chain: Option<Chain>,
    pub symbol: String,
    pub token_id: Option<String>,
    pub network: Option<String>,
    pub enabled: bool,
    pub is_buy_enabled: bool,
    pub is_sell_enabled: bool,
    pub unsupported_countries: Option<HashMap<String, Vec<String>>>,
    pub buy_limits: Vec<FiatAssetLimits>,
    pub sell_limits: Vec<FiatAssetLimits>,
}

impl FiatProviderAsset {
    pub fn asset_id(&self) -> Option<AssetId> {
        match self.clone().chain {
            Some(chain) => match &self.token_id {
                Some(token_id) => format_token_id(chain, token_id.clone()).map(|formatted_token_id| AssetId::from(chain, Some(formatted_token_id))),
                None => Some(chain.as_asset_id()),
            },
            None => None,
        }
    }
}

impl FiatMapping {
    pub fn get_network(network: Option<String>) -> Result<String, crate::error::FiatQuoteError> {
        network.ok_or_else(|| crate::error::FiatQuoteError::InvalidRequest("Missing network".to_string()))
    }
}

pub type FiatMappingMap = HashMap<String, FiatMapping>;

#[derive(Debug, Clone)]
pub struct FiatDeviceContext {
    pub device_id: i32,
    pub wallet_id: i32,
    pub wallet_type: WalletType,
    pub ip_address: String,
    pub country: Option<String>,
}

impl FiatDeviceContext {
    pub fn new(device_id: i32, wallet_id: i32, wallet_type: WalletType, ip_address: String, country: Option<String>) -> Self {
        Self {
            device_id,
            wallet_id,
            wallet_type,
            ip_address,
            country,
        }
    }

    pub fn validate_wallet(&self) -> Result<(), RequestError> {
        match self.wallet_type {
            WalletType::View => Err(RequestError::Forbidden),
            WalletType::Multicoin | WalletType::Single | WalletType::PrivateKey => Ok(()),
        }
    }
}

pub fn filter_token_id(chain: Option<Chain>, token_id: Option<String>) -> Option<String> {
    let token_id = token_id.filter(|contract_address| {
        ![
            "0x0000000000000000000000000000000000001010",
            EVM_ZERO_ADDRESS,
            "0x471ece3750da237f93b8e339c536989b8978a438",
            WORLD_WETH_TOKEN_ID,
            CosmosDenom::Uosmo.as_ref(),
            CosmosDenom::Usei.as_ref(),
            CosmosDenom::Inj.as_ref(),
            CosmosDenom::Uusdc.as_ref(),
            CosmosDenom::Uatom.as_ref(),
            CosmosDenom::Rune.as_ref(),
            CosmosDenom::Utia.as_ref(),
            SOLANA_SYSTEM_PROGRAM_ID,
        ]
        .contains(&contract_address.as_str())
    });
    if let Some(chain) = chain
        && let Some(token_id) = token_id
    {
        return match chain {
            Chain::HyperCore => hypercore_token_id(token_id),
            _ => format_token_id(chain, token_id).filter(|token_id| !AssetId::from_token(chain, token_id).is_native_mirror()),
        };
    }
    token_id
}

fn hypercore_token_id(address: String) -> Option<String> {
    let contract = |asset_id: &AssetId| asset_id.token_components().and_then(|(_, contract, _)| contract);
    if contract(&HYPERCORE_SPOT_HYPE_ASSET_ID).as_ref() == Some(&address) {
        return None;
    }
    [&*HYPERCORE_SPOT_USDC_ASSET_ID, &*HYPERCORE_SPOT_UBTC_ASSET_ID]
        .into_iter()
        .find(|asset_id| contract(asset_id).as_ref() == Some(&address))
        .map_or(Some(address), |asset_id| asset_id.token_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, HYPERCORE_SPOT_UBTC_TOKEN_ID, HYPERCORE_SPOT_USDC_TOKEN_ID};

    #[test]
    fn test_filter_token_id() {
        let filter = |chain: Chain, token_id: &str| filter_token_id(Some(chain), Some(token_id.to_string()));

        assert_eq!(filter(Chain::HyperCore, "0x0d01dc56dcaaca66ad901c959b4011ec"), None);
        assert_eq!(filter(Chain::HyperCore, "0x6d1e7cde53ba9467b783cb7c530ce054"), Some(HYPERCORE_SPOT_USDC_TOKEN_ID.to_string()));
        assert_eq!(filter(Chain::HyperCore, "0x8f254b963e8468305d409b33aa137c67"), Some(HYPERCORE_SPOT_UBTC_TOKEN_ID.to_string()));
        assert_eq!(filter(Chain::HyperCore, "0xfeb2eb8f7d9a2b0f3e2c1f41d2a4c4d3"), Some("0xfeb2eb8f7d9a2b0f3e2c1f41d2a4c4d3".to_string()));
        assert_eq!(filter(Chain::Arc, ARC_USDC_TOKEN_ID), None);
        assert_eq!(filter(Chain::Arc, &ARC_EURC_TOKEN_ID.to_lowercase()), Some(ARC_EURC_TOKEN_ID.to_string()));
    }

    #[test]
    fn test_validate_wallet() {
        assert_eq!(FiatDeviceContext::mock_with_wallet_type(WalletType::View).validate_wallet(), Err(RequestError::Forbidden));
        assert_eq!(FiatDeviceContext::mock().validate_wallet(), Ok(()));
    }
}
