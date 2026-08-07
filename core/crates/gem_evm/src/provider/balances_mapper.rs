use crate::{ethereum_address_checksum, u256::u256_to_biguint};
use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Balance, Chain, hex};
use serde_serializers::biguint_from_hex_str;
use std::error::Error;

pub fn map_balance_coin(balance_hex: String, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
    Ok(AssetBalance::new_balance(chain.as_asset_id(), Balance::coin_balance(biguint_from_hex_str(&balance_hex)?)))
}

fn decode_token_balance(balance_hex: &str) -> Result<BigUint, Box<dyn Error + Send + Sync>> {
    let balance = U256::abi_decode(&hex::decode_hex(balance_hex)?)?;
    Ok(u256_to_biguint(&balance))
}

pub fn map_balance_tokens(balance_data: Vec<String>, token_ids: Vec<String>, chain: Chain) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
    if balance_data.len() != token_ids.len() {
        return Err("Balance data and token IDs length mismatch".into());
    }

    balance_data
        .into_iter()
        .zip(token_ids)
        .map(|(balance_hex, token_id)| {
            let asset_id = AssetId { chain, token_id: Some(token_id) };
            let balance = decode_token_balance(&balance_hex)?;
            Ok(AssetBalance::new_balance(asset_id, Balance::coin_balance(balance)))
        })
        .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()
}

pub fn map_assets_balances(balances: Vec<(String, BigUint)>, chain: Chain) -> Vec<AssetBalance> {
    balances
        .into_iter()
        .filter_map(|(token_address, balance)| {
            if balance.is_zero() {
                return None;
            }

            let checksum_address = ethereum_address_checksum(&token_address).ok()?;
            let asset_id = AssetId::from_token(chain, &checksum_address);
            Some(AssetBalance::new_balance(asset_id, Balance::coin_balance(balance)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let result = map_balance_coin("0x1c6bf52634000".to_string(), Chain::Ethereum).unwrap();
        assert_eq!(result.asset_id.chain, Chain::Ethereum);
        assert_eq!(result.balance.available, BigUint::from(500000000000000_u64));
    }

    #[test]
    fn test_decode_token_balance() {
        for (data, expected) in [
            ("0x000000000000000000000000000000000000000000000000000000000001e240", 123_456_u64),
            (
                concat!(
                    "0x00000000000000000000000000000000000000000000000000000000017faa89",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                ),
                25_143_945_u64,
            ),
        ] {
            assert_eq!(decode_token_balance(data).unwrap(), BigUint::from(expected));
        }
        assert!(decode_token_balance("0x01").is_err());
    }
}
