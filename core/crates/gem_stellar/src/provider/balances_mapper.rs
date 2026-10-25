use crate::constants::{ASSET_TYPE_CREDIT_ALPHANUM4, ASSET_TYPE_CREDIT_ALPHANUM12, ASSET_TYPE_NATIVE, STELLAR_DECIMALS};
use crate::models::account::Account;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &Account) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let chain = Chain::Stellar;
    let reserved_amount = chain.account_activation_fee().unwrap_or(0) as u64;
    let native_balance = account.balances.iter().find(|b| b.asset_type == "native").map(|b| b.balance.clone()).unwrap_or_default();

    let balance_stroops_str = BigNumberFormatter::value_from_amount(&native_balance, STELLAR_DECIMALS)?;
    let balance_decimal = BigNumberFormatter::big_decimal_value(&balance_stroops_str, 0).unwrap_or_default();
    let reserved_decimal = BigNumberFormatter::big_decimal_value(reserved_amount, 0).unwrap_or_default();
    let available_decimal = balance_decimal - reserved_decimal;
    let available = available_decimal.to_string();
    let reserved_str = reserved_amount.to_string();

    let available_biguint = available.parse::<BigUint>().unwrap_or_default();
    let reserved_biguint = reserved_str.parse::<BigUint>().unwrap_or_default();

    Ok(AssetBalance::new_balance(chain.as_asset_id(), Balance::with_reserved(available_biguint, reserved_biguint)))
}

pub fn map_token_balances(account: &Account, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            if let Some((issuer, symbol)) = token_id.split_once("::") {
                if let Some(balance) = account
                    .balances
                    .iter()
                    .find(|b| b.asset_issuer.as_deref() == Some(issuer) && b.asset_code.as_deref() == Some(symbol) && b.asset_type != "native")
                {
                    let amount = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS).unwrap_or_default();
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(amount), true)
                } else {
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(BigUint::from(0u32)), false)
                }
            } else {
                AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(BigUint::from(0u32)), false)
            }
        })
        .collect()
}

pub fn map_all_balances(chain: Chain, account: Account) -> Vec<AssetBalance> {
    account
        .balances
        .into_iter()
        .filter_map(|balance| {
            let asset_id = map_asset_id(chain, &balance.asset_type, balance.asset_issuer.as_ref(), balance.asset_code.as_ref())?;
            let value = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS).ok()?;
            Some(AssetBalance::new_with_active(asset_id, Balance::coin_balance(value), true))
        })
        .collect()
}

pub fn map_asset_id(chain: Chain, asset_type: &str, issuer: Option<&String>, code: Option<&String>) -> Option<AssetId> {
    match (asset_type, issuer, code) {
        (ASSET_TYPE_NATIVE, _, _) => Some(chain.as_asset_id()),
        (ASSET_TYPE_CREDIT_ALPHANUM4 | ASSET_TYPE_CREDIT_ALPHANUM12, Some(issuer), Some(code)) => Some(AssetId::from_token(chain, &AssetId::sub_token_id(&[issuer.clone(), code.clone()]))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;
    use serde_json;

    #[test]
    fn test_map_native_balance() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/balance.json")).unwrap();
        let chain = Chain::Stellar;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(299999077_u64));
        assert_eq!(result.balance.reserved, BigUint::from(10000000_u64));
    }

    #[test]
    fn test_map_native_balance_with_minimal_balance() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let chain = Chain::Stellar;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(0u32));
        assert_eq!(result.balance.reserved, BigUint::from(10000000_u64));
    }
}
