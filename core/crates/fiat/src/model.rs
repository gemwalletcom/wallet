use chain_primitives::format_token_id;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{
    Asset, AssetId, Chain, CosmosDenom, FiatAssetSymbol, FiatProviderName, RequestError, WalletType,
    asset_constants::WORLD_WETH_TOKEN_ID,
    contract_constants::{EVM_ZERO_ADDRESS, SOLANA_SYSTEM_PROGRAM_ID},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FiatMapping {
    pub asset: Asset,
    pub asset_symbol: FiatAssetSymbol,
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
}

impl FiatDeviceContext {
    pub fn new(device_id: i32, wallet_id: i32, wallet_type: WalletType, ip_address: String) -> Self {
        Self {
            device_id,
            wallet_id,
            wallet_type,
            ip_address,
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
    })?;
    if chain.is_some_and(|chain| chain.as_denom() == Some(token_id.as_str())) {
        return None;
    }
    Some(token_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::{APTOS_USDC_ASSET_ID, APTOS_USDC_TOKEN_ID};

    #[test]
    fn test_filter_token_id() {
        assert_eq!(filter_token_id(Some(Chain::Ethereum), Some(EVM_ZERO_ADDRESS.to_string())), None);
        assert_eq!(filter_token_id(Some(Chain::Aptos), Some("0x1::aptos_coin::AptosCoin".to_string())), None);
        assert_eq!(filter_token_id(Some(Chain::Aptos), Some(APTOS_USDC_TOKEN_ID.to_string())), Some(APTOS_USDC_TOKEN_ID.to_string()));
        assert_eq!(filter_token_id(Some(Chain::Cardano), Some("asset1abc".to_string())), Some("asset1abc".to_string()));
        assert_eq!(filter_token_id(None, Some("abc".to_string())), Some("abc".to_string()));
        assert_eq!(filter_token_id(Some(Chain::Aptos), None), None);
    }

    #[test]
    fn test_asset_id() {
        let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

        assert_eq!(FiatProviderAsset::mock().asset_id(), Some(APTOS_USDC_ASSET_ID.clone()));
        assert_eq!(
            FiatProviderAsset {
                chain: Some(Chain::Ethereum),
                token_id: Some(usdc.to_lowercase()),
                ..FiatProviderAsset::mock()
            }
            .asset_id(),
            Some(AssetId::from_token(Chain::Ethereum, usdc))
        );
        assert_eq!(FiatProviderAsset { token_id: None, ..FiatProviderAsset::mock() }.asset_id(), Some(Chain::Aptos.as_asset_id()));
        assert_eq!(
            FiatProviderAsset {
                chain: Some(Chain::Cardano),
                token_id: Some("asset1abc".to_string()),
                ..FiatProviderAsset::mock()
            }
            .asset_id(),
            None
        );
        assert_eq!(FiatProviderAsset { chain: None, ..FiatProviderAsset::mock() }.asset_id(), None);
    }

    #[test]
    fn test_validate_wallet() {
        assert_eq!(FiatDeviceContext::mock_with_wallet_type(WalletType::View).validate_wallet(), Err(RequestError::Forbidden));
        assert_eq!(FiatDeviceContext::mock().validate_wallet(), Ok(()));
    }
}
