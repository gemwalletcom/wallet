#[cfg(test)]
use crate::models::staking::SuiValidator;
use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn map_validators(validators: SuiValidators) -> Vec<DelegationValidator> {
    validators
        .apys
        .into_iter()
        .map(|validator| DelegationValidator::stake(Chain::Sui, validator.address, String::new(), true, 0.0, validator.apy * 100.0))
        .collect()
}

pub fn map_delegations(delegations: Vec<SuiStakeDelegation>, system_state: &SuiSystemState) -> Vec<DelegationBase> {
    delegations
        .into_iter()
        .flat_map(|delegation| {
            let validator_address = delegation.validator_address.clone();
            delegation.stakes.into_iter().map(move |stake| {
                let (state, completion_date) = if stake.stake_active_epoch > system_state.epoch {
                    (DelegationState::Activating, activation_date(stake.stake_active_epoch, system_state))
                } else {
                    (DelegationState::Active, None)
                };

                DelegationBase {
                    asset_id: Chain::Sui.as_asset_id(),
                    state,
                    balance: stake.principal,
                    shares: BigUint::from(0u32),
                    rewards: stake.estimated_reward.unwrap_or_else(|| BigUint::from(0u32)),
                    completion_date,
                    delegation_id: stake.staked_sui_id.clone(),
                    validator_id: validator_address.clone(),
                }
            })
        })
        .collect()
}

pub fn map_staking_apy(validators: SuiValidators) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let max_apy = validators.apys.into_iter().map(|v| v.apy).fold(0.0, f64::max);
    Ok(max_apy * 100.0)
}

fn activation_date(activation_epoch: u64, system_state: &SuiSystemState) -> Option<DateTime<Utc>> {
    let epochs = i64::try_from(activation_epoch.checked_sub(system_state.epoch)?).ok()?;
    let epoch_duration_ms = i64::try_from(system_state.epoch_duration_ms?).ok()?;
    DateTime::from_timestamp_millis(system_state.epoch_start_ms?.checked_add(epochs.checked_mul(epoch_duration_ms)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_delegations() {
        let delegations: Vec<SuiStakeDelegation> = serde_json::from_str(include_str!("../../testdata/stakes.json")).unwrap();

        let result = map_delegations(delegations.clone(), &SuiSystemState::mock());
        assert_eq!(
            result.iter().map(|delegation| delegation.state).collect::<Vec<_>>(),
            vec![DelegationState::Active, DelegationState::Active, DelegationState::Active, DelegationState::Activating, DelegationState::Active]
        );
        assert_eq!(result[3].completion_date, DateTime::from_timestamp_millis(1_750_086_400_000));
        assert_eq!(result[0].completion_date, None);

        let result = map_delegations(
            delegations.clone(),
            &SuiSystemState {
                epoch_duration_ms: None,
                ..SuiSystemState::mock()
            },
        );
        assert_eq!(result[3].state, DelegationState::Activating);
        assert_eq!(result[3].completion_date, None);

        let result = map_delegations(delegations, &SuiSystemState { epoch: 860, ..SuiSystemState::mock() });
        assert_eq!(result.iter().map(|delegation| delegation.state).collect::<Vec<_>>(), vec![DelegationState::Active; 5]);
    }

    #[test]
    fn test_map_validators_uses_individual_apys() {
        let validators = map_validators(SuiValidators {
            apys: vec![
                SuiValidator {
                    address: "validator1".to_string(),
                    apy: 0.015625,
                },
                SuiValidator {
                    address: "validator2".to_string(),
                    apy: 0.03125,
                },
            ],
        });

        assert_eq!(
            validators.into_iter().map(|validator| (validator.id, validator.apr)).collect::<Vec<_>>(),
            vec![("validator1".to_string(), 1.5625), ("validator2".to_string(), 3.125)]
        );
    }
}
