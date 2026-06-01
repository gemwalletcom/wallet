use crate::models::Balance as SuiBalance;
use crate::models::staking::SuiStakeDelegation;
use crate::{coin_type_matches, is_sui_coin};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_balance_coin(balance: SuiBalance) -> AssetBalance {
    AssetBalance::new_balance(
        Chain::Sui.as_asset_id(),
        Balance::coin_balance(BigUint::try_from(balance.total_balance).unwrap_or_default()),
    )
}

pub fn map_balance_tokens(balances: Vec<SuiBalance>, token_ids: Vec<String>) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            let balance = balances
                .iter()
                .find(|b| coin_type_matches(&b.coin_type, &token_id))
                .map(|b| &b.total_balance)
                .cloned()
                .unwrap_or_default();

            AssetBalance::new_balance(
                AssetId::from_token(Chain::Sui, &token_id),
                Balance::coin_balance(BigUint::try_from(balance).unwrap_or_default()),
            )
        })
        .collect()
}

pub fn map_balance_staking(delegations: Vec<SuiStakeDelegation>) -> AssetBalance {
    let staked = delegations
        .iter()
        .flat_map(|delegation| &delegation.stakes)
        .map(|stake| &stake.principal + stake.estimated_reward.as_ref().unwrap_or(&BigUint::from(0u32)))
        .sum::<BigUint>();

    AssetBalance::new_balance(Chain::Sui.as_asset_id(), Balance::stake_balance(staked, BigUint::from(0u32), None))
}

fn map_token_asset_balance(balance: SuiBalance) -> Option<AssetBalance> {
    if is_sui_coin(&balance.coin_type) {
        return None;
    }

    Some(AssetBalance::new_balance(
        AssetId::from_token(Chain::Sui, &balance.coin_type),
        Balance::coin_balance(BigUint::try_from(balance.total_balance).unwrap_or_default()),
    ))
}

pub fn map_assets_balances(balances: Vec<SuiBalance>) -> Vec<AssetBalance> {
    balances.into_iter().filter_map(map_token_asset_balance).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::SUI_USDC_TOKEN_ID;

    #[test]
    fn test_map_coin_balance() {
        let balance: SuiBalance = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();

        let result = map_balance_coin(balance);
        assert_eq!(result.balance.available, BigUint::from(52855428706_u64));
        assert_eq!(result.asset_id.chain, Chain::Sui);
    }

    #[test]
    fn test_map_token_balances() {
        let balances: Vec<SuiBalance> = serde_json::from_str(include_str!("../../testdata/balance_tokens.json")).unwrap();

        let token_ids = vec![
            SUI_USDC_TOKEN_ID.to_string(),
            "0xda1644f58a955833a15abae24f8cc65b5bd8152ce013fde8be0a6a3dcf51fe36::token::TOKEN".to_string(),
        ];

        let result = map_balance_tokens(balances, token_ids);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].balance.available, BigUint::from(3685298_u64)); // USDC balance
        assert_eq!(result[1].balance.available, BigUint::from(1000_u64)); // TOKEN balance
    }

    #[test]
    fn test_coin_type_matches() {
        assert!(coin_type_matches("0x2::sui::SUI", "0x2::sui::SUI"));
        assert!(coin_type_matches("0x2::sui::SUI", "2::sui::SUI"));
        assert!(coin_type_matches("2::sui::SUI", "0x2::sui::SUI"));
        assert!(!coin_type_matches("0x2::sui::SUI", "0x3::token::TOKEN"));
    }

    #[test]
    fn test_map_balance_staking() {
        let delegations: Vec<SuiStakeDelegation> = serde_json::from_str(include_str!("../../testdata/stakes.json")).unwrap();

        let balance = map_balance_staking(delegations);

        assert_eq!(balance.asset_id.chain, Chain::Sui);

        assert_eq!(balance.balance.staked, BigUint::from(9113484503_u64));
        assert_eq!(balance.balance.available, BigUint::from(0u32));
    }

    #[test]
    fn test_map_balance_staking_empty() {
        let delegations: Vec<SuiStakeDelegation> = vec![];
        let balance = map_balance_staking(delegations);

        assert_eq!(balance.asset_id.chain, Chain::Sui);
        assert_eq!(balance.balance.staked, BigUint::from(0u32));
        assert_eq!(balance.balance.available, BigUint::from(0u32));
    }

    #[test]
    fn test_map_assets_balances() {
        let balances: Vec<SuiBalance> = serde_json::from_str(include_str!("../../testdata/balance_tokens.json")).unwrap();

        let result = map_assets_balances(balances);

        assert_eq!(result.len(), 7);
        assert_eq!(
            result[0].asset_id,
            AssetId::from_token(Chain::Sui, "0xce7ff77a83ea0cb6fd39bd8748e2ec89a3f41e8efdc3f4eb123e0ca37b184db2::buck::BUCK")
        );
        assert_eq!(result[1].asset_id, AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID));
        assert_eq!(result[1].balance.available, BigUint::from(3685298_u64));
        assert_eq!(
            result[3].asset_id,
            AssetId::from_token(Chain::Sui, "0xda1644f58a955833a15abae24f8cc65b5bd8152ce013fde8be0a6a3dcf51fe36::token::TOKEN")
        );
        assert_eq!(result[3].balance.available, BigUint::from(1000_u64));
        assert_eq!(result.iter().filter(|balance| balance.asset_id == Chain::Sui.as_asset_id()).count(), 0);
    }
}
