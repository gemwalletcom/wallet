use crate::address::TronAddress;
use crate::models::{TronAccount, TronReward, TronUnfrozen, TronVote, WitnessesList};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{Address as _, Asset, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn map_staking_delegations(account: TronAccount, reward: TronReward, validators: &[DelegationValidator], now: DateTime<Utc>) -> Vec<DelegationBase> {
    let asset_id = Chain::Tron.as_asset_id();
    let unfreezes = account.unfrozen_v2.map(|unfrozen| map_unfreeze_delegations(unfrozen, &asset_id, now)).unwrap_or_default();
    let votes = account
        .votes
        .map(|votes| map_vote_delegations(votes, reward.reward, validators, &asset_id))
        .unwrap_or_default();

    unfreezes.into_iter().chain(votes).collect()
}

fn map_vote_delegations(votes: Vec<TronVote>, reward: u64, validators: &[DelegationValidator], asset_id: &AssetId) -> Vec<DelegationBase> {
    let total_votes: u64 = votes.iter().map(|vote| vote.vote_count).sum();
    let decimals = 10_u64.pow(Asset::from_chain(Chain::Tron).decimals as u32);

    votes
        .into_iter()
        .filter(|vote| validators.iter().any(|validator| validator.id == vote.vote_address))
        .map(|vote| {
            let proportional_reward = if total_votes > 0 {
                (reward as f64 * vote.vote_count as f64 / total_votes as f64) as u64
            } else {
                0
            };
            DelegationBase {
                asset_id: asset_id.clone(),
                state: DelegationState::Active,
                balance: BigUint::from(vote.vote_count * decimals),
                shares: BigUint::from(vote.vote_count),
                rewards: BigUint::from(proportional_reward),
                completion_date: None,
                delegation_id: format!("vote_{}", vote.vote_address),
                validator_id: vote.vote_address,
            }
        })
        .collect()
}

pub fn map_unfreeze_delegations(unfrozen: Vec<TronUnfrozen>, asset_id: &AssetId, now: DateTime<Utc>) -> Vec<DelegationBase> {
    let (expired, pending): (Vec<_>, Vec<_>) = unfrozen
        .into_iter()
        .filter_map(|item| {
            let completion_date = DateTime::from_timestamp((item.unfreeze_expire_time? / 1000) as i64, 0).unwrap_or(now);
            Some((completion_date, item.unfreeze_amount))
        })
        .partition(|(completion_date, _)| *completion_date <= now);

    let withdrawable = expired.iter().map(|(completion_date, _)| *completion_date).max().map(|completion_date| {
        unfreeze_delegation(
            asset_id,
            DelegationState::AwaitingWithdrawal,
            expired.iter().map(|(_, amount)| BigUint::from(*amount)).sum(),
            completion_date,
        )
    });

    pending
        .into_iter()
        .map(|(completion_date, amount)| unfreeze_delegation(asset_id, DelegationState::Pending, BigUint::from(amount), completion_date))
        .chain(withdrawable)
        .collect()
}

fn unfreeze_delegation(asset_id: &AssetId, state: DelegationState, balance: BigUint, completion_date: DateTime<Utc>) -> DelegationBase {
    DelegationBase {
        asset_id: asset_id.clone(),
        state,
        balance,
        shares: BigUint::from(0u32),
        rewards: BigUint::from(0u32),
        completion_date: Some(completion_date),
        delegation_id: completion_date.timestamp().to_string(),
        validator_id: DelegationValidator::SYSTEM_ID.to_string(),
    }
}

pub fn map_staking_validators(witnesses: WitnessesList, apy: Option<f64>) -> Vec<DelegationValidator> {
    let default_apy = apy.unwrap_or(0.0);
    let mut validators: Vec<DelegationValidator> = witnesses
        .witnesses
        .into_iter()
        .filter_map(|witness| {
            Some(DelegationValidator::stake(
                Chain::Tron,
                TronAddress::from_hex(&witness.address)?.encode(),
                String::new(),
                witness.is_jobs.unwrap_or(false),
                0.0,
                default_apy,
            ))
        })
        .collect();

    validators.push(DelegationValidator::system(Chain::Tron));

    validators
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WitnessAccount;

    fn unfrozen(amount: u64, expire_time: u64) -> TronUnfrozen {
        TronUnfrozen {
            unfreeze_amount: amount,
            unfreeze_expire_time: Some(expire_time),
        }
    }

    #[test]
    fn test_every_expired_unfreeze_is_one_withdrawal() {
        let now = DateTime::from_timestamp(1_000_000, 0).unwrap();
        let asset_id = Chain::Tron.as_asset_id();

        let delegations = map_unfreeze_delegations(vec![unfrozen(6, 900_000_000), unfrozen(4, 800_000_000)], &asset_id, now);

        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(delegations[0].balance, BigUint::from(10u32));
    }

    #[test]
    fn test_an_unfreeze_still_locked_stays_on_its_own() {
        let now = DateTime::from_timestamp(1_000_000, 0).unwrap();
        let asset_id = Chain::Tron.as_asset_id();

        let delegations = map_unfreeze_delegations(vec![unfrozen(6, 900_000_000), unfrozen(4, 2_000_000_000)], &asset_id, now);

        assert_eq!(delegations.len(), 2);
        assert_eq!(delegations[0].state, DelegationState::Pending);
        assert_eq!(delegations[0].balance, BigUint::from(4u32));
        assert_eq!(delegations[1].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(delegations[1].balance, BigUint::from(6u32));
    }

    fn create_mock_witnesses() -> WitnessesList {
        WitnessesList {
            witnesses: vec![
                WitnessAccount {
                    address: "4159f3440fd40722f716144e4490a4de162d3b3fcb".to_string(),
                    vote_count: Some(1000000),
                    url: "https://validator1.com".to_string(),
                    is_jobs: Some(true),
                },
                WitnessAccount {
                    address: "41357a7401a0f0c2d4a44a1881a0c622f15d986291".to_string(),
                    vote_count: Some(500000),
                    url: "https://validator2.com".to_string(),
                    is_jobs: Some(false),
                },
            ],
        }
    }

    #[test]
    fn test_map_staking_validators() {
        let witnesses = create_mock_witnesses();
        let validators = map_staking_validators(witnesses, Some(4.2));

        assert_eq!(validators.len(), 3);

        assert_eq!(validators[0].chain, Chain::Tron);
        assert_eq!(validators[0].id, "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        assert_eq!(validators[0].name, "");
        assert!(validators[0].is_active);
        assert_eq!(validators[0].commission, 0.0);
        assert_eq!(validators[0].apr, 4.2);

        assert_eq!(validators[1].id, "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1");
        assert!(!validators[1].is_active);

        assert_eq!(validators[2].id, DelegationValidator::SYSTEM_ID);
        assert_eq!(validators[2].name, DelegationValidator::SYSTEM_NAME);
        assert!(validators[2].is_active);
    }
}
