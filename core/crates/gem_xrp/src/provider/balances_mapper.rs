use crate::models::rpc::{AccountInfo, AccountObjects};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::{collections::HashMap, error::Error};

pub fn map_balance_coin(account: Option<AccountInfo>, asset_id: AssetId, base_reserve: u64, owner_reserve: u64) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let (available, reserved) = account
        .map(|account| {
            let reserved = base_reserve + u64::from(account.owner_count) * owner_reserve;
            (account.balance.saturating_sub(reserved), reserved)
        })
        .unwrap_or_default();

    Ok(AssetBalance::new_balance(asset_id, Balance::with_reserved(BigUint::from(available), BigUint::from(reserved))))
}

fn account_objects_to_balances(objects: &AccountObjects, address: &str, chain: Chain) -> Vec<AssetBalance> {
    objects
        .account_objects
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|obj| {
            if obj.high_limit.currency.len() <= 3 {
                return None;
            }
            let (issuer, value) = if obj.low_limit.issuer == address {
                (&obj.high_limit.issuer, obj.balance.value.clone())
            } else if obj.high_limit.issuer == address {
                (&obj.low_limit.issuer, -obj.balance.value.clone())
            } else {
                return None;
            };

            Some(AssetBalance::new_with_active(AssetId::from_token(chain, issuer), Balance::coin_balance(value.to_biguint().unwrap_or_default()), true))
        })
        .collect()
}

pub fn map_balance_tokens(objects: &AccountObjects, address: &str, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    let available_balances: HashMap<String, AssetBalance> = account_objects_to_balances(objects, address, chain)
        .into_iter()
        .filter_map(|x| x.asset_id.token_id.clone().map(|token_id| (token_id, x)))
        .collect();

    token_ids
        .into_iter()
        .map(|token_id| {
            available_balances.get(&token_id).cloned().unwrap_or_else(|| {
                let asset_id = AssetId::from_token(chain, &token_id);
                let balance = Balance::coin_balance(BigUint::from(0u32));
                AssetBalance::new_with_active(asset_id, balance, false)
            })
        })
        .collect()
}

pub fn map_balance_assets(objects: &AccountObjects, address: &str, chain: Chain) -> Vec<AssetBalance> {
    account_objects_to_balances(objects, address, chain).into_iter().filter(|x| x.balance.available > BigUint::from(0u32)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::result::XRPResult;
    use crate::models::rpc::AccountInfo;
    use primitives::{AssetId, Chain};

    const FIXTURE_ADDRESS: &str = "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297";
    const RLUSD_CURRENCY: &str = "524C555344000000000000000000000000000000";

    #[test]
    fn test_map_balance_coin() {
        let asset_id = AssetId::from_chain(Chain::Xrp);
        let base_reserve = 1_000_000;
        let owner_reserve = 200_000;

        let with_owned_objects = map_balance_coin(Some(AccountInfo::mock_with_balance(35_892_065, 2)), asset_id.clone(), base_reserve, owner_reserve).unwrap();
        assert_eq!(with_owned_objects.asset_id, asset_id);
        assert_eq!(with_owned_objects.balance.available, BigUint::from(34_492_065_u64));
        assert_eq!(with_owned_objects.balance.reserved, BigUint::from(1_400_000_u64));

        let without_owned_objects = map_balance_coin(Some(AccountInfo::mock_with_balance(10_000_000, 0)), asset_id.clone(), base_reserve, owner_reserve).unwrap();
        assert_eq!(without_owned_objects.balance.available, BigUint::from(9_000_000_u64));
        assert_eq!(without_owned_objects.balance.reserved, BigUint::from(1_000_000_u64));

        let balance_below_reserve = map_balance_coin(Some(AccountInfo::mock_with_balance(500_000, 2)), asset_id.clone(), base_reserve, owner_reserve).unwrap();
        assert_eq!(balance_below_reserve.balance.available, BigUint::ZERO);
        assert_eq!(balance_below_reserve.balance.reserved, BigUint::from(1_400_000_u64));
    }

    #[test]
    fn test_map_balance_tokens() {
        let response: XRPResult<AccountObjects> = serde_json::from_str(include_str!("../testdata/accounts_objects_tokens.json")).unwrap();
        let account_objects = response.result;

        let token_ids = vec!["rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string()];

        let result = map_balance_tokens(&account_objects, FIXTURE_ADDRESS, token_ids, Chain::Xrp);

        assert_eq!(result.len(), 1);

        let balance = &result[0];
        assert_eq!(balance.asset_id, AssetId::from_token(Chain::Xrp, "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"));
        assert_eq!(balance.balance.available, BigUint::from(171000000000000_u64));
        assert!(balance.is_active);
    }

    #[test]
    fn test_map_balance_assets() {
        let response: XRPResult<AccountObjects> = serde_json::from_str(include_str!("../testdata/accounts_objects_tokens.json")).unwrap();
        let account_objects = response.result;

        let result = map_balance_assets(&account_objects, FIXTURE_ADDRESS, Chain::Xrp);

        assert!(!result.is_empty());
        for balance in &result {
            assert_eq!(balance.asset_id.chain, Chain::Xrp);
            assert!(balance.asset_id.token_id.is_some());
            assert!(balance.balance.available > BigUint::from(0u32));
            assert!(balance.is_active);
        }
    }

    #[test]
    fn test_map_balance_tokens_trust_line_sides() {
        let issuer = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De";
        let objects = |low: &str, high: &str, balance: &str| -> AccountObjects {
            serde_json::from_value(serde_json::json!({
                "account_objects": [{
                    "LowLimit": { "currency": RLUSD_CURRENCY, "issuer": low },
                    "HighLimit": { "currency": RLUSD_CURRENCY, "issuer": high },
                    "Balance": { "currency": RLUSD_CURRENCY, "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji", "value": balance }
                }]
            }))
            .unwrap()
        };
        let holder = "rsHolderHighSide";

        let fresh = map_balance_tokens(&objects(FIXTURE_ADDRESS, issuer, "0"), FIXTURE_ADDRESS, vec![issuer.to_string()], Chain::Xrp);
        let high = map_balance_tokens(&objects(issuer, holder, "-5"), holder, vec![issuer.to_string()], Chain::Xrp);

        assert!(fresh[0].is_active);
        assert_eq!(fresh[0].balance.available, BigUint::ZERO);
        assert!(high[0].is_active);
        assert_eq!(high[0].balance.available, BigUint::from(5_000_000_000_000_000u64));
    }
}
