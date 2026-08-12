use std::error::Error;

use num_bigint::BigUint;
use primitives::{Asset, AssetBalance, AssetId, AssetType, Chain};

use crate::models::FungibleTokenMetadata;

pub fn map_token_balance(token_id: &str, value: &str) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let balance = value.parse::<BigUint>()?;
    Ok(AssetBalance::new(AssetId::from_token(Chain::Near, token_id), balance))
}

pub fn map_token_data(token_id: &str, metadata: FungibleTokenMetadata) -> Asset {
    Asset::new(
        AssetId::from_token(Chain::Near, token_id),
        metadata.name,
        metadata.symbol,
        metadata.decimals.into(),
        AssetType::TOKEN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_token_responses() {
        let balance = map_token_balance("token.near", "1234567").unwrap();
        assert_eq!(balance.asset_id, AssetId::from_token(Chain::Near, "token.near"));
        assert_eq!(balance.balance.available, BigUint::from(1_234_567u64));

        let metadata = FungibleTokenMetadata {
            name: "Example Token".to_string(),
            symbol: "EXT".to_string(),
            decimals: 8,
        };
        let asset = map_token_data("token.near", metadata);
        assert_eq!(asset.id, AssetId::from_token(Chain::Near, "token.near"));
        assert_eq!(asset.name, "Example Token");
        assert_eq!(asset.symbol, "EXT");
        assert_eq!(asset.decimals, 8);
        assert_eq!(asset.asset_type, AssetType::TOKEN);
    }

    #[test]
    fn test_map_token_balance_rejects_invalid_amount() {
        assert_eq!(map_token_balance("token.near", "invalid").unwrap_err().to_string(), "invalid digit found in string");
    }
}
