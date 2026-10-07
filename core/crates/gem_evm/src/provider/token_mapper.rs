use crate::{contracts::erc20::decode_token_metadata, ethereum_address_checksum};
use primitives::{Asset, AssetId, Chain};

pub fn map_token_data(chain: Chain, token_id: String, name_hex: String, symbol_hex: String, decimals_hex: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
    let (name, symbol, decimals) = decode_token_metadata(&name_hex, &symbol_hex, &decimals_hex)?;
    let token_id = ethereum_address_checksum(&token_id)?;

    let asset_id = AssetId { chain, token_id: Some(token_id) };

    let asset_type = asset_id.chain.default_asset_type().ok_or("Invalid token metadata: chain has no token asset type")?;

    Ok(Asset::new(asset_id, name, symbol, decimals.into(), asset_type))
}

pub fn map_is_token_address(token_id: &str) -> bool {
    token_id.starts_with("0x") && token_id.len() == 42
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TOKEN_USDC_ADDRESS;
    use primitives::AssetType;

    #[test]
    fn test_map_is_token_address() {
        assert!(map_is_token_address(TOKEN_USDC_ADDRESS));
        assert!(!map_is_token_address("0x1234"));
        assert!(!map_is_token_address(&format!("{TOKEN_USDC_ADDRESS}123")));
        assert!(!map_is_token_address(TOKEN_USDC_ADDRESS.trim_start_matches("0x")));
        assert!(!map_is_token_address(""));
        assert!(!map_is_token_address("0x"));
    }

    #[test]
    fn test_map_token_data() {
        let token_id = TOKEN_USDC_ADDRESS.to_ascii_lowercase();
        let chain = Chain::Ethereum;
        let name_hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000855534420436f696e000000000000000000000000000000000000000000000000".to_string();
        let symbol_hex = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000045553444300000000000000000000000000000000000000000000000000000000".to_string();
        let decimals_hex = "0x0000000000000000000000000000000000000000000000000000000000000006".to_string();

        let result = map_token_data(chain, token_id, name_hex, symbol_hex, decimals_hex).unwrap();

        assert_eq!(result.name, "USD Coin");
        assert_eq!(result.symbol, "USDC");
        assert_eq!(result.decimals, 6);
        assert_eq!(result.id.chain, Chain::Ethereum);
        assert_eq!(result.chain(), Chain::Ethereum);
        assert_eq!(result.token_id(), Some(TOKEN_USDC_ADDRESS));
        assert_eq!(result.asset_type, AssetType::ERC20);
    }

    #[test]
    fn test_map_token_data_missing_metadata() {
        let token_id = TOKEN_USDC_ADDRESS.to_ascii_lowercase();
        let name_hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000855534420436f696e000000000000000000000000000000000000000000000000".to_string();
        let symbol_hex = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000045553444300000000000000000000000000000000000000000000000000000000".to_string();
        let decimals_hex = "0x0000000000000000000000000000000000000000000000000000000000000006".to_string();

        let result = map_token_data(Chain::Ethereum, token_id.clone(), String::new(), symbol_hex.clone(), decimals_hex.clone()).unwrap();

        assert_eq!(result.name, "USDC");
        assert_eq!(result.symbol, "USDC");
        assert!(map_token_data(Chain::Ethereum, token_id.clone(), name_hex.clone(), String::new(), decimals_hex.clone()).is_err());
        assert!(map_token_data(Chain::Ethereum, token_id.clone(), String::new(), String::new(), decimals_hex).is_err());
        assert!(map_token_data(Chain::Ethereum, token_id.clone(), name_hex.clone(), symbol_hex.clone(), "0x".to_string()).is_err());
        assert!(map_token_data(Chain::Ethereum, token_id, name_hex, symbol_hex, String::new()).is_err());
    }
}
