use crate::models::{
    balance::{Balance as SpotBalance, Balances, StakeBalance},
    token::SpotToken,
};
use number_formatter::BigNumberFormatter;
use primitives::{Asset, AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_balance_token(asset_id: AssetId, balance: &SpotBalance, available_after_maintenance: Option<&str>, decimals: i32) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let decimals = decimals as u32;
    let total = BigNumberFormatter::value_from_amount_biguint(&balance.total, decimals)?;
    let available = match available_after_maintenance {
        Some(amount) => BigNumberFormatter::value_from_amount_biguint(amount, decimals)?,
        None => {
            let hold = BigNumberFormatter::value_from_amount_biguint(&balance.hold, decimals)?;
            total.clone() - hold.min(total.clone())
        }
    };
    let available = available.min(total.clone());
    let reserved = total - available.clone();

    Ok(AssetBalance::new_balance(asset_id, Balance::with_reserved(available, reserved)))
}

pub fn map_balance_assets(spot_balances: &Balances, spot_tokens: &[SpotToken], chain: Chain) -> Vec<AssetBalance> {
    spot_balances
        .balances
        .iter()
        .filter_map(|x| {
            let token = spot_tokens.iter().find(|t| t.index as u32 == x.token)?;
            map_balance_token(token.asset_id(chain), x, spot_balances.available_after_maintenance(x.token), token.wei_decimals).ok()
        })
        .collect()
}

pub fn map_balance_tokens(spot_balances: &Balances, spot_tokens: &[SpotToken], token_ids: &[String], chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .iter()
        .filter_map(|token_id| {
            let asset_id = AssetId::from(chain, Some(token_id.clone()));
            let token = spot_tokens.iter().find(|token| token_matches(token, &asset_id))?;
            if let Some(balance) = spot_balances.balances.iter().find(|b| b.token == token.index as u32) {
                map_balance_token(asset_id, balance, spot_balances.available_after_maintenance(balance.token), token.wei_decimals).ok()
            } else {
                Some(AssetBalance::new_zero_balance(asset_id))
            }
        })
        .collect()
}

fn token_matches(token: &SpotToken, asset_id: &AssetId) -> bool {
    let Some((symbol, contract, index)) = asset_id.token_components() else {
        return false;
    };
    token.name == symbol && contract.is_none_or(|contract| contract == token.token_id) && index.is_none_or(|index| index == token.index)
}

pub fn map_balance_staking(balance: &StakeBalance, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let native_decimals = Asset::from_chain(chain).decimals as u32;
    let available_biguint = BigNumberFormatter::value_from_amount_biguint(&balance.delegated, native_decimals).unwrap_or_default();
    let pending_biguint = BigNumberFormatter::value_from_amount_biguint(&balance.total_pending_withdrawal, native_decimals).unwrap_or_default();

    Ok(AssetBalance::new_balance(chain.as_asset_id(), Balance::stake_balance(available_biguint, pending_biguint, None)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::Balance;
    use num_bigint::BigUint;
    use primitives::{Chain, asset_constants::HYPERCORE_SPOT_USDC_TOKEN_ID};

    #[test]
    fn test_map_balance_token() {
        let asset_id = AssetId::from(Chain::HyperCore, Some("USDC::0".to_string()));
        let balance = SpotBalance {
            coin: "USDC".to_string(),
            token: 0,
            total: "12.89353003".to_string(),
            hold: "2.020032".to_string(),
        };

        let result = map_balance_token(asset_id.clone(), &balance, None, 8).unwrap();

        assert_eq!(result.balance.available, "1087349803".parse::<BigUint>().unwrap());
        assert_eq!(result.balance.reserved, "202003200".parse::<BigUint>().unwrap());
        assert_eq!(result.asset_id.token_id, Some("USDC::0".to_string()));

        let result = map_balance_token(asset_id, &balance, Some("12.69152703"), 8).unwrap();

        assert_eq!(result.balance.available, "1269152703".parse::<BigUint>().unwrap());
        assert_eq!(result.balance.reserved, "20200300".parse::<BigUint>().unwrap());
    }

    #[test]
    fn test_map_balance_tokens() {
        let spot_balances = Balances {
            token_to_available_after_maintenance: vec![],
            balances: vec![Balance {
                coin: "USDC".to_string(),
                token: 0,
                total: "56003537".to_string(),
                hold: "0".to_string(),
            }],
        };

        let spot_tokens = vec![SpotToken {
            name: "USDC".to_string(),
            wei_decimals: 8,
            index: 0,
            token_id: "0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
            sz_decimals: 2,
        }];

        let token_ids_by_symbol = vec!["USDC".to_string()];
        let results = map_balance_tokens(&spot_balances, &spot_tokens, &token_ids_by_symbol, Chain::HyperCore);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id.chain, Chain::HyperCore);
        assert_eq!(results[0].balance.available, "5600353700000000".parse::<BigUint>().unwrap());

        let token_ids_full = vec![HYPERCORE_SPOT_USDC_TOKEN_ID.to_string()];
        let results_full = map_balance_tokens(&spot_balances, &spot_tokens, &token_ids_full, Chain::HyperCore);

        assert_eq!(results_full.len(), 1);
        assert_eq!(results_full[0].asset_id.chain, Chain::HyperCore);
        assert_eq!(results_full[0].balance.available, "5600353700000000".parse::<BigUint>().unwrap());
    }

    #[test]
    fn test_a_balance_stays_with_the_token_identity_that_was_asked_for() {
        let first = SpotToken {
            name: "USDC".to_string(),
            wei_decimals: 8,
            index: 0,
            token_id: "0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
            sz_decimals: 2,
        };
        let impostor = SpotToken {
            name: "USDC".to_string(),
            wei_decimals: 8,
            index: 7,
            token_id: "0xdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
            sz_decimals: 2,
        };
        let spot_balances = Balances {
            token_to_available_after_maintenance: vec![],
            balances: vec![
                Balance {
                    coin: "USDC".to_string(),
                    token: 0,
                    total: "1".to_string(),
                    hold: "0".to_string(),
                },
                Balance {
                    coin: "USDC".to_string(),
                    token: 7,
                    total: "9".to_string(),
                    hold: "0".to_string(),
                },
            ],
        };
        let requested = vec![format!("USDC::{}::7", impostor.token_id)];

        for spot_tokens in [vec![first.clone(), impostor.clone()], vec![impostor.clone(), first.clone()]] {
            let results = map_balance_tokens(&spot_balances, &spot_tokens, &requested, Chain::HyperCore);

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].asset_id.token_id, Some(requested[0].clone()));
            assert_eq!(
                results[0].balance.available,
                "900000000".parse::<BigUint>().unwrap(),
                "the balance belongs to the token that was asked for, whatever order the metadata arrived in"
            );
        }

        let unknown = vec![format!("USDC::{}::3", impostor.token_id)];
        assert!(
            map_balance_tokens(&spot_balances, &[first, impostor], &unknown, Chain::HyperCore).is_empty(),
            "an identity no token has is not answered with another token's balance"
        );
    }

    #[test]
    fn test_map_balance_tokens_missing_balance() {
        let spot_balances = Balances {
            balances: vec![],
            token_to_available_after_maintenance: vec![],
        };

        let spot_tokens = vec![SpotToken {
            name: "USDC".to_string(),
            wei_decimals: 8,
            index: 0,
            token_id: "0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
            sz_decimals: 2,
        }];

        let token_ids = vec!["USDC".to_string()];
        let results = map_balance_tokens(&spot_balances, &spot_tokens, &token_ids, Chain::HyperCore);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id.chain, Chain::HyperCore);
        assert_eq!(results[0].balance.available, BigUint::from(0u64));
    }

    #[test]
    fn test_map_balance_staking() {
        let stake_balance = StakeBalance {
            delegated: "100.0".to_string(),
            undelegated: "0.0".to_string(),
            total_pending_withdrawal: "10.0".to_string(),
        };
        let result = map_balance_staking(&stake_balance, Chain::HyperCore).unwrap();

        assert_eq!(result.balance.staked, BigUint::from(10_000_000_000u64));
        assert_eq!(result.balance.pending, BigUint::from(1_000_000_000u64));
    }
}
