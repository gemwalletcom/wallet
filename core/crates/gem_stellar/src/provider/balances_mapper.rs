use crate::constants::{ACCOUNT_BASE_RESERVES, ASSET_TYPE_CREDIT_ALPHANUM4, ASSET_TYPE_CREDIT_ALPHANUM12, ASSET_TYPE_NATIVE, BASE_RESERVE, STELLAR_DECIMALS};
use crate::models::account::{Account, Balance as AccountBalance};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &Account) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let native = account.balances.iter().find(|b| b.asset_type == ASSET_TYPE_NATIVE);
    let balance = BigNumberFormatter::value_from_amount_biguint(native.map(|b| b.balance.as_str()).unwrap_or_default(), STELLAR_DECIMALS)?;
    let selling_liabilities = native.map(map_selling_liabilities).transpose()?.unwrap_or_default();
    let reserves = (ACCOUNT_BASE_RESERVES + u64::from(account.subentry_count) + u64::from(account.num_sponsoring)).saturating_sub(u64::from(account.num_sponsored));
    let reserved = BigUint::from(reserves * BASE_RESERVE) + selling_liabilities;
    let available = if balance > reserved { balance - &reserved } else { BigUint::ZERO };

    Ok(AssetBalance::new_balance(Chain::Stellar.as_asset_id(), Balance::with_reserved(available, reserved)))
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
                    let balance = map_token_balance(balance).unwrap_or_else(|_| Balance::coin_balance(BigUint::ZERO));
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), balance, true)
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

fn map_token_balance(balance: &AccountBalance) -> Result<Balance, Box<dyn Error + Sync + Send>> {
    let amount = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS)?;
    let selling_liabilities = map_selling_liabilities(balance)?;
    let available = if amount > selling_liabilities { &amount - &selling_liabilities } else { BigUint::ZERO };
    let reserved = amount - &available;
    Ok(Balance::with_reserved(available, reserved))
}

fn map_selling_liabilities(balance: &AccountBalance) -> Result<BigUint, Box<dyn Error + Sync + Send>> {
    let value = balance.selling_liabilities.as_deref().map(|value| BigNumberFormatter::value_from_amount_biguint(value, STELLAR_DECIMALS));
    Ok(value.transpose()?.unwrap_or_default())
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
    use primitives::asset_constants::{STELLAR_USDC_ASSET_ID, STELLAR_USDC_TOKEN_ID};

    #[test]
    fn test_map_native_balance() {
        let trustline: Account = serde_json::from_str(include_str!("../../testdata/balance.json")).unwrap();
        let minimal: Account = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let sponsoring: Account = serde_json::from_str(include_str!("../../testdata/balance_sponsoring.json")).unwrap();
        let sponsored: Account = serde_json::from_str(include_str!("../../testdata/balance_sponsored.json")).unwrap();
        let selling_offer: Account = serde_json::from_str(include_str!("../../testdata/balance_selling_offer.json")).unwrap();
        let without_counts: Account = serde_json::from_str(r#"{"sequence":"1","balances":[{"balance":"1.5000000","asset_type":"native"}],"thresholds":{"med_threshold":0},"signers":[]}"#).unwrap();
        let asset_id = Chain::Stellar.as_asset_id();

        assert_eq!(
            map_native_balance(&trustline).unwrap(),
            AssetBalance::new_balance(asset_id.clone(), Balance::with_reserved(BigUint::from(294_999_077_u64), BigUint::from(15_000_000_u64)))
        );
        assert_eq!(
            map_native_balance(&minimal).unwrap(),
            AssetBalance::new_balance(asset_id.clone(), Balance::with_reserved(BigUint::ZERO, BigUint::from(10_000_000_u64)))
        );
        assert_eq!(
            map_native_balance(&sponsoring).unwrap(),
            AssetBalance::new_balance(asset_id.clone(), Balance::with_reserved(BigUint::from(3_669_209_334_u64), BigUint::from(740_000_000_u64)))
        );
        assert_eq!(
            map_native_balance(&sponsored).unwrap(),
            AssetBalance::new_balance(asset_id.clone(), Balance::with_reserved(BigUint::from(737_762_u64), BigUint::ZERO))
        );
        assert_eq!(
            map_native_balance(&selling_offer).unwrap(),
            AssetBalance::new_balance(asset_id.clone(), Balance::with_reserved(BigUint::from(521_181_719_u64), BigUint::from(152_705_008_913_u64)))
        );
        assert_eq!(
            map_native_balance(&without_counts).unwrap(),
            AssetBalance::new_balance(asset_id, Balance::with_reserved(BigUint::from(5_000_000_u64), BigUint::from(10_000_000_u64)))
        );
    }

    #[test]
    fn test_map_token_balances() {
        let without_offer: Account = serde_json::from_str(include_str!("../../testdata/balance.json")).unwrap();
        let selling_offer: Account = serde_json::from_str(include_str!("../../testdata/balance_selling_offer.json")).unwrap();
        let token_ids = vec![STELLAR_USDC_TOKEN_ID.to_string()];
        let asset_id = STELLAR_USDC_ASSET_ID.clone();

        assert_eq!(
            map_token_balances(&without_offer, token_ids.clone(), Chain::Stellar),
            vec![AssetBalance::new_with_active(asset_id.clone(), Balance::coin_balance(BigUint::from(276_243_772_u64)), true)]
        );
        assert_eq!(
            map_token_balances(&selling_offer, token_ids, Chain::Stellar),
            vec![AssetBalance::new_with_active(asset_id, Balance::with_reserved(BigUint::from(21_690_460_327_u64), BigUint::from(70_785_091_889_u64)), true)]
        );
    }
}
