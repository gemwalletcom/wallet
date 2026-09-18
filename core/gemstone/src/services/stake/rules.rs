use std::collections::{HashMap, HashSet};

use crate::services::collections::{stale, unique};

use num_bigint::{BigInt, BigUint};
use primitives::AddressName;
use primitives::{
    AddressFormatStyle, AddressFormatter, AddressType, Asset, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, EarnType, RedelegateData, Resource,
    StakeChain, StakeProviderType, StakeType, VerificationStatus, WalletType, YieldProvider,
};
use rand::seq::IndexedRandom;
use std::str::FromStr;

use super::model::{
    GemClaimRewards, GemClaimRewardsDestination, GemDelegationAction, GemDelegationAmountInput, GemDelegationCompletion, GemDelegationDestination, GemDelegationRow,
    GemDelegationStatus, GemDelegationTone, GemStakeAction, GemStakeActionItem, GemStakeAmountInput, GemStakeInfoRow, GemStakeSection, GemStakeValidatorSelection, GemValidatorRow,
};
use crate::config::image::GemImage;
use crate::duration_formatter::{GemDurationPart, countdown_parts, day_parts};
use chrono::{DateTime, Utc};
use crate::config::stake::EARN_OFFERED;
use crate::models::custom_types::GemBigUint;
use crate::services::balance::{GemAssetBalance, GemBalanceRow};
use crate::services::error::GemServiceError;
use crate::services::transfer::rules as transfer_rules;

use crate::config::chain::account_activation_fee_url;
use crate::config::stake::{StakeChainConfig, get_stake_config};
use crate::config::validators::get_validators;

pub fn delegation_destination(wallet_type: WalletType, asset: Asset, delegation: Delegation) -> GemDelegationDestination {
    if wallet_type == WalletType::View || delegation.base.state != DelegationState::AwaitingWithdrawal {
        return GemDelegationDestination::Details;
    }
    delegation_action_destination(asset, delegation, GemDelegationAction::Withdraw, vec![])
}

pub fn delegation_action_destination(asset: Asset, delegation: Delegation, action: GemDelegationAction, validators: Vec<DelegationValidator>) -> GemDelegationDestination {
    let value = BigInt::from(delegation.base.balance.clone());
    let stake = |asset, input| GemDelegationDestination::Amount {
        asset,
        input: GemDelegationAmountInput::Stake { input },
    };
    let earn = |asset, earn_type| GemDelegationDestination::Amount {
        asset,
        input: GemDelegationAmountInput::Earn { earn_type },
    };
    let confirm = move |asset, stake_type| GemDelegationDestination::Confirm {
        transfer: transfer_rules::stake_transfer_data(asset, stake_type, value, false),
    };
    match action {
        GemDelegationAction::Stake => stake(
            asset,
            GemStakeAmountInput::Stake {
                validators,
                validator: Some(delegation.validator),
            },
        ),
        GemDelegationAction::Redelegate => stake(
            asset,
            GemStakeAmountInput::Redelegate {
                validators,
                delegation,
                validator: None,
            },
        ),
        GemDelegationAction::Unstake if can_change_amount_on_unstake(asset.chain()) => stake(asset, GemStakeAmountInput::Unstake { delegation }),
        GemDelegationAction::Unstake => confirm(asset, StakeType::Unstake(delegation)),
        GemDelegationAction::Withdraw => match delegation.validator.provider_type {
            StakeProviderType::Stake => confirm(asset, StakeType::Withdraw(delegation)),
            StakeProviderType::Earn => earn(asset, EarnType::Withdraw(delegation)),
        },
        GemDelegationAction::Deposit => earn(asset, EarnType::Deposit(delegation.validator)),
    }
}

pub fn delegation_actions(wallet_type: WalletType, delegation: &Delegation) -> Vec<GemDelegationAction> {
    if wallet_type == WalletType::View {
        return vec![];
    }
    let state = delegation.base.state;
    match delegation.validator.provider_type {
        StakeProviderType::Stake => {
            let Some(config) = stake_config(delegation.base.asset_id.chain) else {
                return vec![];
            };
            match state {
                DelegationState::Active if config.can_redelegate => vec![GemDelegationAction::Stake, GemDelegationAction::Unstake, GemDelegationAction::Redelegate],
                DelegationState::Active => vec![GemDelegationAction::Unstake],
                DelegationState::Inactive if config.can_redelegate => vec![GemDelegationAction::Unstake, GemDelegationAction::Redelegate],
                DelegationState::Inactive => vec![GemDelegationAction::Unstake],
                DelegationState::AwaitingWithdrawal if config.can_withdraw => vec![GemDelegationAction::Withdraw],
                DelegationState::AwaitingWithdrawal | DelegationState::Pending | DelegationState::Activating | DelegationState::Deactivating => vec![],
            }
        }
        StakeProviderType::Earn => match state {
            DelegationState::Active => vec![GemDelegationAction::Deposit, GemDelegationAction::Withdraw],
            DelegationState::Inactive => vec![GemDelegationAction::Withdraw],
            DelegationState::Pending | DelegationState::Activating | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal => vec![],
        },
    }
}

pub fn earn_apr(providers: &[DelegationValidator], asset_apr: Option<f64>) -> f64 {
    providers.first().map(|provider| provider.apr).filter(|apr| *apr > 0.0).or(asset_apr).unwrap_or_default()
}

pub fn can_claim_rewards(wallet_type: WalletType, delegation: &Delegation) -> bool {
    let Some(config) = stake_config(delegation.base.asset_id.chain) else {
        return false;
    };
    wallet_type != WalletType::View && config.can_claim_rewards && shows_rewards(&delegation.base)
}

fn validator_display_name(validator: &DelegationValidator) -> String {
    if validator.name.is_empty() {
        return AddressFormatter::format(&validator.id, Some(validator.chain), AddressFormatStyle::Short);
    }
    validator.name.clone()
}

pub fn validator_row(validator: &DelegationValidator) -> GemValidatorRow {
    let name = validator_display_name(validator);
    let provider = match validator.provider_type {
        StakeProviderType::Earn => YieldProvider::from_str(&validator.id).ok(),
        StakeProviderType::Stake => None,
    };
    GemValidatorRow {
        image_url: GemImage::Validator {
            chain: validator.chain,
            validator_id: validator.id.clone(),
        }
        .url(),
        placeholder: name.chars().next().map(String::from).unwrap_or_default(),
        name,
        provider,
        validator: validator.clone(),
    }
}

pub fn validator_explorer_address(validator: &DelegationValidator) -> Option<String> {
    match validator.provider_type {
        StakeProviderType::Stake if !DelegationValidator::is_system_id(&validator.id) => Some(validator.id.clone()),
        StakeProviderType::Stake | StakeProviderType::Earn => None,
    }
}

pub fn delegation_status(delegation: &Delegation) -> GemDelegationStatus {
    let state = match delegation.base.state {
        DelegationState::Active if !delegation.validator.is_active => DelegationState::Inactive,
        state => state,
    };
    GemDelegationStatus {
        state,
        tone: delegation_tone(state),
        completion: delegation_completion(delegation),
    }
}

fn delegation_tone(state: DelegationState) -> GemDelegationTone {
    match state {
        DelegationState::Active => GemDelegationTone::Positive,
        DelegationState::Pending | DelegationState::Activating | DelegationState::Deactivating => GemDelegationTone::Pending,
        DelegationState::Inactive | DelegationState::AwaitingWithdrawal => GemDelegationTone::Negative,
    }
}

fn delegation_completion(delegation: &Delegation) -> Option<GemDelegationCompletion> {
    match delegation.validator.provider_type {
        StakeProviderType::Earn => None,
        StakeProviderType::Stake => match delegation.base.state {
            DelegationState::Activating => Some(GemDelegationCompletion::ActiveIn),
            DelegationState::Pending | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal => Some(GemDelegationCompletion::AvailableIn),
            DelegationState::Active | DelegationState::Inactive => None,
        },
    }
}

pub fn shows_rewards(delegation: &DelegationBase) -> bool {
    delegation.state == DelegationState::Active && delegation.rewards > BigUint::ZERO
}

pub fn requires_frozen_balance(chain: Chain, frozen_value: &BigUint) -> bool {
    uses_freeze(chain) && *frozen_value == BigUint::ZERO
}

fn can_claim_stake_rewards(chain: Chain, rewards_value: &BigUint) -> bool {
    stake_config(chain).is_some_and(|config| config.can_claim_rewards) && *rewards_value > BigUint::ZERO
}

pub fn can_claim_all_rewards(chain: Chain, delegations_with_rewards: usize) -> bool {
    stake_config(chain).is_some_and(|config| config.can_claim_all_rewards) || delegations_with_rewards == 1
}

fn stake_config(chain: Chain) -> Option<StakeChainConfig> {
    StakeChain::from_chain(chain).map(get_stake_config)
}

pub fn stake_sections(uses_freeze: bool, has_actions: bool, has_delegations: bool) -> Vec<GemStakeSection> {
    [
        has_actions.then_some(GemStakeSection::Manage),
        uses_freeze.then_some(GemStakeSection::Resources),
        has_delegations.then_some(GemStakeSection::Delegations),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn stake_info_rows(chain: Chain, staking_apr: Option<f64>) -> Vec<GemStakeInfoRow> {
    [
        staking_apr.filter(|apr| *apr != 0.0).map(|_| GemStakeInfoRow::Apr),
        (lock_time_seconds(chain) > 0).then_some(GemStakeInfoRow::LockTime),
        (min_stake_amount(chain) > BigInt::ZERO).then_some(GemStakeInfoRow::MinimumAmount),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn delegation_rows(delegation: &Delegation) -> Vec<GemDelegationRow> {
    [
        Some(GemDelegationRow::Provider),
        (delegation.validator.apr != 0.0).then_some(GemDelegationRow::Apr),
        Some(GemDelegationRow::Status),
        delegation_status(delegation).completion.map(|_| GemDelegationRow::CompletionDate),
        shows_rewards(&delegation.base).then_some(GemDelegationRow::Rewards),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn sorted_delegations(mut delegations: Vec<Delegation>) -> Vec<Delegation> {
    delegations.sort_by(|a, b| b.base.balance.cmp(&a.base.balance));
    delegations
}

pub fn positions(delegations: Vec<Delegation>) -> Vec<Delegation> {
    delegations.into_iter().filter(|delegation| delegation.base.balance > BigUint::ZERO).collect()
}

pub fn lock_time_seconds(chain: Chain) -> u64 {
    stake_config(chain).map(|config| config.time_lock).unwrap_or_default()
}

pub fn lock_time_parts(chain: Chain) -> Vec<GemDurationPart> {
    day_parts(lock_time_seconds(chain) as i64)
}

pub fn completion_countdown_parts(delegation: &Delegation, now: DateTime<Utc>) -> Vec<GemDurationPart> {
    let Some(completion_date) = delegation.base.completion_date else {
        return vec![];
    };
    if delegation_status(delegation).completion.is_none() {
        return vec![];
    }
    let remaining = (completion_date - now).num_seconds();
    if remaining <= 0 {
        return vec![];
    }
    countdown_parts(remaining)
}

pub fn min_stake_amount(chain: Chain) -> BigInt {
    stake_config(chain).map(|config| BigInt::from(config.min_amount)).unwrap_or_default()
}

fn can_change_amount_on_unstake(chain: Chain) -> bool {
    stake_config(chain).is_some_and(|config| config.change_amount_on_unstake)
}

pub fn uses_freeze(chain: Chain) -> bool {
    stake_config(chain).is_some_and(|config| config.uses_freeze)
}

pub fn uses_whole_amounts(chain: Chain) -> bool {
    stake_config(chain).is_some_and(|config| config.uses_whole_amounts)
}

pub fn rewards_value(delegations: &[Delegation]) -> BigUint {
    delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum()
}

pub fn stake_actions(wallet_type: WalletType, chain: Chain, has_validators: bool, balance: &GemAssetBalance, delegations: &[Delegation]) -> Vec<GemStakeActionItem> {
    let Some(config) = stake_config(chain).filter(|_| wallet_type != WalletType::View) else {
        return vec![];
    };
    let uses_freeze = config.uses_freeze;
    let requires_frozen_balance = requires_frozen_balance(chain, &(&balance.frozen + &balance.locked));
    let item = |action: GemStakeAction, is_enabled: bool, requires_frozen_balance: bool| GemStakeActionItem {
        action,
        is_enabled,
        requires_frozen_balance,
    };
    [
        Some(item(GemStakeAction::Stake, has_validators || requires_frozen_balance, requires_frozen_balance)),
        uses_freeze.then(|| item(GemStakeAction::Freeze, true, false)),
        uses_freeze.then(|| item(GemStakeAction::Unfreeze, true, false)),
        can_claim_stake_rewards(chain, &rewards_value(delegations)).then(|| item(GemStakeAction::ClaimRewards, true, false)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn claim_rewards(chain: Chain, delegations: Vec<Delegation>) -> GemClaimRewards {
    let with_rewards: Vec<Delegation> = delegations.into_iter().filter(|delegation| delegation.base.rewards > BigUint::ZERO).collect();
    let value = BigInt::from(rewards_value(&with_rewards));
    let destination = if can_claim_all_rewards(chain, with_rewards.len()) {
        let validators = with_rewards.into_iter().map(|delegation| delegation.validator).collect();
        GemClaimRewardsDestination::Transfer {
            transfer: transfer_rules::stake_transfer_data(Asset::from_chain(chain), StakeType::Rewards(validators), value.clone(), false),
        }
    } else {
        GemClaimRewardsDestination::Amount { delegations: with_rewards }
    };
    GemClaimRewards { value, destination }
}

#[uniffi::export]
impl GemAssetBalance {
    pub fn staked_value(&self, chain: Chain) -> GemBigUint {
        let principal = if uses_freeze(chain) { &self.frozen + &self.locked } else { self.staked.clone() };
        principal + &self.pending + &self.rewards
    }

    pub fn detail_rows(&self, chain: Chain, is_stake_enabled: bool) -> Vec<GemBalanceRow> {
        let positive = |value: &GemBigUint| (*value > GemBigUint::ZERO).then(|| value.clone());
        let rows: Vec<GemBalanceRow> = [
            self.shows_stake_balance(chain, is_stake_enabled)
                .then(|| GemBalanceRow::Staked { value: self.staked_value(chain) }),
            positive(&self.earn).filter(|_| EARN_OFFERED).map(|value| GemBalanceRow::Earn { value }),
            positive(&self.pending_unconfirmed).map(|value| GemBalanceRow::PendingUnconfirmed { value }),
            positive(&self.reserved).map(|value| GemBalanceRow::Reserved {
                value,
                url: account_activation_fee_url(chain),
            }),
        ]
        .into_iter()
        .flatten()
        .collect();
        let held_beyond_available = [
            &self.frozen,
            &self.locked,
            &self.pending,
            &self.pending_unconfirmed,
            &self.staked,
            &self.rewards,
            &self.reserved,
            &self.earn,
        ]
        .into_iter()
        .any(|value| *value > GemBigUint::ZERO);
        if held_beyond_available {
            std::iter::once(GemBalanceRow::Available { value: self.available.clone() }).chain(rows).collect()
        } else {
            rows
        }
    }
}

impl GemAssetBalance {
    fn shows_stake_balance(&self, chain: Chain, is_stake_enabled: bool) -> bool {
        StakeChain::from_chain(chain).is_some() && (is_stake_enabled || self.staked_value(chain) > GemBigUint::ZERO)
    }
}

pub fn selectable_validators(validators: Vec<DelegationValidator>) -> Vec<DelegationValidator> {
    let mut selectable: Vec<DelegationValidator> = validators
        .into_iter()
        .filter(|validator| validator.is_active && !validator.name.trim().is_empty() && !DelegationValidator::is_system_id(&validator.id))
        .collect();
    selectable.sort_by(|left, right| right.apr.total_cmp(&left.apr));
    selectable
}

fn recommended_validator_ids(chain: Chain) -> Vec<String> {
    get_validators().remove(chain.as_ref()).unwrap_or_default()
}

fn recommended_validators(chain: Chain, validators: &[DelegationValidator]) -> Vec<DelegationValidator> {
    let recommended = recommended_validator_ids(chain);
    validators.iter().filter(|validator| recommended.contains(&validator.id)).cloned().collect()
}

fn recommended_validator(chain: Chain, validators: Vec<DelegationValidator>) -> Option<DelegationValidator> {
    recommended_validators(chain, &validators)
        .choose(&mut rand::rng())
        .cloned()
        .or_else(|| validators.first().cloned())
}

fn other_validators(validators: &[DelegationValidator], validator_id: &str) -> Vec<DelegationValidator> {
    validators.iter().filter(|validator| validator.id != validator_id).cloned().collect()
}

pub fn validator_selection(chain: Chain, input: &GemStakeAmountInput) -> GemStakeValidatorSelection {
    let rows = |validators: Vec<DelegationValidator>| validators.iter().map(validator_row).collect();
    match input {
        GemStakeAmountInput::Stake { validators, validator } => GemStakeValidatorSelection {
            options: rows(validators.clone()),
            recommended: rows(recommended_validators(chain, validators)),
            validator: validator.clone().or_else(|| recommended_validator(chain, validators.clone())).as_ref().map(validator_row),
            can_select: true,
        },
        GemStakeAmountInput::Redelegate {
            validators,
            delegation,
            validator,
        } => GemStakeValidatorSelection {
            options: rows(validators.clone()),
            recommended: rows(recommended_validators(chain, &other_validators(validators, &delegation.validator.id))),
            validator: validator
                .clone()
                .or_else(|| recommended_validator(chain, other_validators(validators, &delegation.validator.id)))
                .as_ref()
                .map(validator_row),
            can_select: true,
        },
        GemStakeAmountInput::Unstake { delegation } | GemStakeAmountInput::Withdraw { delegation } => GemStakeValidatorSelection {
            options: vec![validator_row(&delegation.validator)],
            recommended: vec![],
            validator: Some(validator_row(&delegation.validator)),
            can_select: false,
        },
        GemStakeAmountInput::Rewards { delegations, validator } => GemStakeValidatorSelection {
            options: delegations.iter().map(|delegation| validator_row(&delegation.validator)).collect(),
            recommended: vec![],
            validator: rewards_validator(delegations, validator).as_ref().map(validator_row),
            can_select: delegations.len() > 1,
        },
        GemStakeAmountInput::Freeze { .. } | GemStakeAmountInput::Unfreeze { .. } => GemStakeValidatorSelection {
            options: Vec::new(),
            recommended: vec![],
            validator: None,
            can_select: false,
        },
    }
}

pub(crate) fn rewards_validator(delegations: &[Delegation], validator: &Option<DelegationValidator>) -> Option<DelegationValidator> {
    validator.clone().or_else(|| delegations.first().map(|delegation| delegation.validator.clone()))
}

pub fn stake_type(input: &GemStakeAmountInput) -> Result<StakeType, GemServiceError> {
    match input {
        GemStakeAmountInput::Stake { validator, .. } => Ok(StakeType::Stake(confirmed_validator(validator)?)),
        GemStakeAmountInput::Redelegate { delegation, validator, .. } => Ok(StakeType::Redelegate(RedelegateData {
            delegation: delegation.clone(),
            to_validator: confirmed_validator(validator)?,
        })),
        GemStakeAmountInput::Unstake { delegation } => Ok(StakeType::Unstake(delegation.clone())),
        GemStakeAmountInput::Withdraw { delegation } => Ok(StakeType::Withdraw(delegation.clone())),
        GemStakeAmountInput::Rewards { validator, .. } => Ok(StakeType::Rewards(vec![confirmed_validator(validator)?])),
        GemStakeAmountInput::Freeze { resource } => Ok(StakeType::Freeze(*resource)),
        GemStakeAmountInput::Unfreeze { resource } => Ok(StakeType::Unfreeze(*resource)),
    }
}

fn confirmed_validator(validator: &Option<DelegationValidator>) -> Result<DelegationValidator, GemServiceError> {
    validator.clone().ok_or_else(|| GemServiceError::InvalidInput {
        msg: "stake needs a validator".to_string(),
    })
}

pub fn with_validator(input: &GemStakeAmountInput, validator: DelegationValidator) -> GemStakeAmountInput {
    match input {
        GemStakeAmountInput::Stake { validators, .. } => GemStakeAmountInput::Stake {
            validators: validators.clone(),
            validator: Some(validator),
        },
        GemStakeAmountInput::Redelegate { validators, delegation, .. } => GemStakeAmountInput::Redelegate {
            validators: validators.clone(),
            delegation: delegation.clone(),
            validator: Some(validator),
        },
        GemStakeAmountInput::Rewards { delegations, .. } => GemStakeAmountInput::Rewards {
            delegations: delegations.clone(),
            validator: Some(validator),
        },
        GemStakeAmountInput::Unstake { .. } | GemStakeAmountInput::Withdraw { .. } | GemStakeAmountInput::Freeze { .. } | GemStakeAmountInput::Unfreeze { .. } => input.clone(),
    }
}

pub fn with_resource(input: &GemStakeAmountInput, resource: Resource) -> GemStakeAmountInput {
    match input {
        GemStakeAmountInput::Freeze { .. } => GemStakeAmountInput::Freeze { resource },
        GemStakeAmountInput::Unfreeze { .. } => GemStakeAmountInput::Unfreeze { resource },
        GemStakeAmountInput::Stake { .. }
        | GemStakeAmountInput::Redelegate { .. }
        | GemStakeAmountInput::Unstake { .. }
        | GemStakeAmountInput::Withdraw { .. }
        | GemStakeAmountInput::Rewards { .. } => input.clone(),
    }
}

pub fn merge_validators(validators: Vec<DelegationValidator>, delegation_validators: Vec<DelegationValidator>, names: &HashMap<String, String>) -> Vec<DelegationValidator> {
    let active_ids: HashSet<String> = validators.iter().map(|validator| validator.id.clone()).collect();
    validators
        .into_iter()
        .chain(delegation_validators.into_iter().filter(|validator| !active_ids.contains(&validator.id)))
        .map(|mut validator| {
            if validator.name.is_empty() {
                validator.name = names.get(&validator.id).cloned().unwrap_or_default();
            }
            validator
        })
        .collect()
}

fn inactive_validator(chain: Chain, id: String, name: String) -> DelegationValidator {
    DelegationValidator {
        chain,
        id,
        name,
        is_active: false,
        commission: 0.0,
        apr: 0.0,
        provider_type: StakeProviderType::Stake,
    }
}

pub fn missing_validators(
    chain: Chain,
    delegations: &[DelegationBase],
    existing: &HashMap<String, DelegationValidator>,
    names: &HashMap<String, String>,
) -> Vec<DelegationValidator> {
    unique(delegations.iter().map(|delegation| delegation.validator_id.clone()).filter(|id| !existing.contains_key(id)))
        .into_iter()
        .map(|id| {
            let name = names.get(&id).filter(|name| !name.is_empty()).cloned().unwrap_or_else(|| id.clone());
            inactive_validator(chain, id, name)
        })
        .collect()
}

pub fn delegations_with_state(delegations: Vec<DelegationBase>, validators: &HashMap<String, DelegationValidator>) -> Vec<DelegationBase> {
    delegations
        .into_iter()
        .map(|mut delegation| {
            if let Some(validator) = validators.get(&delegation.validator_id)
                && delegation.state == DelegationState::Active
                && !validator.is_active
            {
                delegation.state = DelegationState::Inactive;
            }
            delegation
        })
        .collect()
}

pub fn stale_delegation_ids(existing_ids: Vec<String>, incoming: &[DelegationBase]) -> Vec<String> {
    stale(existing_ids, incoming.iter().map(DelegationBase::id))
}

pub fn stale_validator_ids(existing: Vec<DelegationValidator>, incoming: &[DelegationValidator]) -> Vec<String> {
    stale(
        existing.into_iter().filter(|validator| validator.is_active).map(|validator| validator.id),
        incoming.iter().map(|validator| validator.id.clone()),
    )
}

pub fn validator_address_names(validators: &[DelegationValidator]) -> Vec<AddressName> {
    validators
        .iter()
        .filter(|validator| !validator.name.is_empty())
        .map(|validator| AddressName {
            chain: validator.chain,
            address: validator.id.clone(),
            name: validator.name.clone(),
            address_type: AddressType::Validator,
            status: VerificationStatus::Verified,
            image_url: None,
        })
        .collect()
}

pub fn earn_validators(providers: Vec<DelegationValidator>, apr: f64) -> Vec<DelegationValidator> {
    providers.into_iter().map(|provider| DelegationValidator { apr, ..provider }).collect()
}

#[cfg(test)]
mod tests {
    use crate::duration_formatter::GemDurationUnit;
    use chrono::Duration;
    use super::*;
    use crate::services::transfer::GemTransferData;
    use primitives::Resource;

    #[test]
    fn test_validator_display_name() {
        let solana = DelegationValidator {
            chain: Chain::Solana,
            id: "8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD".to_string(),
            name: String::new(),
            ..DelegationValidator::mock()
        };
        assert_eq!(validator_display_name(&DelegationValidator::mock()), "Test Validator");
        assert_eq!(validator_display_name(&solana), "8GbwA...JF8iD");

        let earn = DelegationValidator {
            id: "yo".to_string(),
            name: String::new(),
            provider_type: StakeProviderType::Earn,
            ..DelegationValidator::mock()
        };
        assert_eq!(validator_display_name(&earn), "yo");
    }

    #[test]
    fn test_a_validator_row_names_the_validator_and_picks_its_image() {
        let row = validator_row(&DelegationValidator {
            chain: Chain::Solana,
            id: "8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD".to_string(),
            name: String::new(),
            ..DelegationValidator::mock()
        });
        assert_eq!(row.name, "8GbwA...JF8iD");
        assert_eq!(row.placeholder, "8");
        assert!(
            row.image_url.contains("8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD"),
            "a staking validator draws its own logo"
        );
        assert_eq!(row.provider, None);

        let earn = DelegationValidator {
            id: "yo".to_string(),
            provider_type: StakeProviderType::Earn,
            ..DelegationValidator::mock()
        };
        assert_eq!(validator_row(&earn).provider, Some(YieldProvider::Yo));

        let unknown = DelegationValidator {
            id: "not-a-provider".to_string(),
            provider_type: StakeProviderType::Earn,
            ..DelegationValidator::mock()
        };
        assert_eq!(validator_row(&unknown).provider, None, "an unknown yield provider falls back to the logo");
    }

    #[test]
    fn test_validator_address_names() {
        let named = DelegationValidator {
            chain: Chain::Solana,
            id: "8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD".to_string(),
            name: "Everstake".to_string(),
            ..DelegationValidator::mock()
        };

        let names = validator_address_names(&[
            named.clone(),
            DelegationValidator {
                name: String::new(),
                ..named.clone()
            },
        ]);

        assert_eq!(names.len(), 1);
        assert_eq!(names[0].name, "Everstake");
        assert_eq!(names[0].address, named.id);
    }

    #[test]
    fn test_a_chain_without_staking_answers_instead_of_failing() {
        assert_eq!(lock_time_seconds(Chain::Bitcoin), 0);
        assert_eq!(min_stake_amount(Chain::Bitcoin), BigInt::ZERO);
        assert!(!can_change_amount_on_unstake(Chain::Bitcoin));
        assert!(!uses_freeze(Chain::Bitcoin));
        assert!(!uses_whole_amounts(Chain::Bitcoin));

        assert!(uses_freeze(Chain::Tron));
        assert!(uses_whole_amounts(Chain::Tron));
        assert!(lock_time_seconds(Chain::Sui) > 0);
        assert!(min_stake_amount(Chain::Sui) > BigInt::ZERO);
    }

    #[test]
    fn test_validator_explorer_address_skips_system_and_earn_validators() {
        assert_eq!(
            validator_explorer_address(&DelegationValidator::mock_cosmos("cosmosvaloper1")),
            Some("cosmosvaloper1".to_string())
        );
        assert_eq!(validator_explorer_address(&DelegationValidator::mock_cosmos(DelegationValidator::SYSTEM_ID)), None);
        assert_eq!(validator_explorer_address(&DelegationValidator::mock_cosmos("unstaking")), None);
        assert_eq!(
            validator_explorer_address(&DelegationValidator {
                provider_type: StakeProviderType::Earn,
                ..DelegationValidator::mock_cosmos("cosmosvaloper1")
            }),
            None
        );
    }

    #[test]
    fn test_the_stake_screen_shows_only_the_info_rows_its_chain_has() {
        assert_eq!(
            stake_info_rows(Chain::Cosmos, Some(0.0)),
            vec![GemStakeInfoRow::LockTime],
            "an apr of zero is not an apr row"
        );
        assert_eq!(stake_info_rows(Chain::Cosmos, None), vec![GemStakeInfoRow::LockTime]);
        assert_eq!(stake_info_rows(Chain::Cosmos, Some(12.5)), vec![GemStakeInfoRow::Apr, GemStakeInfoRow::LockTime]);
        assert_eq!(
            stake_info_rows(Chain::Ethereum, Some(3.0)),
            vec![GemStakeInfoRow::Apr, GemStakeInfoRow::LockTime, GemStakeInfoRow::MinimumAmount],
            "a chain with a minimum states it"
        );
    }

    #[test]
    fn test_the_stake_sections_follow_what_the_chain_and_wallet_offer() {
        assert_eq!(
            stake_sections(true, true, true),
            vec![GemStakeSection::Manage, GemStakeSection::Resources, GemStakeSection::Delegations]
        );
        assert_eq!(
            stake_sections(false, false, true),
            vec![GemStakeSection::Delegations],
            "a chain without freezing and a wallet with no actions keeps only its delegations"
        );
        assert!(stake_sections(false, false, false).is_empty(), "an empty wallet has no delegations section to title");
    }

    #[test]
    fn test_a_delegation_shows_its_completion_and_rewards_rows_only_when_it_has_them() {
        let active = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 100);
        assert_eq!(
            delegation_rows(&active),
            vec![GemDelegationRow::Provider, GemDelegationRow::Apr, GemDelegationRow::Status, GemDelegationRow::Rewards]
        );

        let pending = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Pending, 0);
        assert!(delegation_rows(&pending).contains(&GemDelegationRow::CompletionDate));
        assert!(!delegation_rows(&pending).contains(&GemDelegationRow::Rewards), "no rewards row without rewards");

        let no_apr = Delegation {
            validator: DelegationValidator {
                apr: 0.0,
                ..active.validator.clone()
            },
            ..active
        };
        assert!(!delegation_rows(&no_apr).contains(&GemDelegationRow::Apr));
    }

    #[test]
    fn test_the_lock_time_reads_as_whole_days() {
        assert_eq!(
            lock_time_parts(Chain::Cosmos),
            vec![GemDurationPart {
                value: (lock_time_seconds(Chain::Cosmos) / 86_400) as i64,
                unit: GemDurationUnit::Day
            }]
        );
    }

    #[test]
    fn test_a_delegation_counts_down_to_its_completion_only_while_one_is_pending() {
        let now = Utc::now();
        let mut pending = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Deactivating, 0);
        pending.base.completion_date = Some(now + Duration::days(2));
        let mut active = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0);
        active.base.completion_date = Some(now + Duration::days(2));

        assert_eq!(completion_countdown_parts(&pending, now).first().map(|part| (part.value, part.unit)), Some((2, GemDurationUnit::Day)));
        assert!(completion_countdown_parts(&active, now).is_empty());
        pending.base.completion_date = Some(now - Duration::hours(1));
        assert!(completion_countdown_parts(&pending, now).is_empty());
    }

    #[test]
    fn test_delegation_status_presents_the_state_its_tone_and_the_completion_label() {
        let status = |state, provider_type| delegation_status(&Delegation::mock_with(Chain::Cosmos, provider_type, state, 100));

        let active = status(DelegationState::Active, StakeProviderType::Stake);
        assert_eq!((active.state, active.tone, active.completion), (DelegationState::Active, GemDelegationTone::Positive, None));
        let inactive = status(DelegationState::Inactive, StakeProviderType::Stake);
        assert_eq!(
            (inactive.state, inactive.tone, inactive.completion),
            (DelegationState::Inactive, GemDelegationTone::Negative, None)
        );
        let activating = status(DelegationState::Activating, StakeProviderType::Stake);
        assert_eq!(
            (activating.tone, activating.completion),
            (GemDelegationTone::Pending, Some(GemDelegationCompletion::ActiveIn))
        );
        for state in [DelegationState::Pending, DelegationState::Deactivating] {
            let pending = status(state, StakeProviderType::Stake);
            assert_eq!(
                (pending.state, pending.tone, pending.completion),
                (state, GemDelegationTone::Pending, Some(GemDelegationCompletion::AvailableIn))
            );
        }
        let awaiting = status(DelegationState::AwaitingWithdrawal, StakeProviderType::Stake);
        assert_eq!(
            (awaiting.tone, awaiting.completion),
            (GemDelegationTone::Negative, Some(GemDelegationCompletion::AvailableIn))
        );
        assert_eq!(status(DelegationState::Pending, StakeProviderType::Earn).completion, None);

        let mut on_inactive_validator = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 100);
        on_inactive_validator.validator.is_active = false;
        let shown = delegation_status(&on_inactive_validator);
        assert_eq!((shown.state, shown.tone), (DelegationState::Inactive, GemDelegationTone::Negative));
    }

    #[test]
    fn test_delegation_rows_follow_state() {
        for state in [
            DelegationState::Pending,
            DelegationState::Activating,
            DelegationState::Deactivating,
            DelegationState::AwaitingWithdrawal,
        ] {
            let base = Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, state, 100).base;
            assert!(!shows_rewards(&base));
        }
        assert!(shows_rewards(
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 100).base
        ));
        assert!(!shows_rewards(
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0).base
        ));
        assert!(!shows_rewards(
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Inactive, 100).base
        ));
    }

    fn destination_kind(destination: &GemDelegationDestination) -> &'static str {
        match destination {
            GemDelegationDestination::Details => "details",
            GemDelegationDestination::Confirm { .. } => "confirm",
            GemDelegationDestination::Amount {
                input: GemDelegationAmountInput::Stake { input },
                ..
            } => match input {
                GemStakeAmountInput::Stake { .. } => "stake amount",
                GemStakeAmountInput::Redelegate { .. } => "redelegate amount",
                GemStakeAmountInput::Unstake { .. } => "unstake amount",
                GemStakeAmountInput::Withdraw { .. } | GemStakeAmountInput::Rewards { .. } | GemStakeAmountInput::Freeze { .. } | GemStakeAmountInput::Unfreeze { .. } => {
                    "other stake amount"
                }
            },
            GemDelegationDestination::Amount {
                input: GemDelegationAmountInput::Earn { earn_type },
                ..
            } => match earn_type {
                EarnType::Deposit(_) => "earn deposit amount",
                EarnType::Withdraw(_) => "earn withdraw amount",
            },
        }
    }

    fn confirm_transfer(destination: GemDelegationDestination) -> GemTransferData {
        match destination {
            GemDelegationDestination::Confirm { transfer } => transfer,
            GemDelegationDestination::Details | GemDelegationDestination::Amount { .. } => panic!("expected a confirm transfer"),
        }
    }

    #[test]
    fn test_a_delegation_tap_withdraws_only_an_awaiting_withdrawal_on_a_signing_wallet() {
        let asset = Asset::from_chain(Chain::Solana);
        let awaiting = Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::AwaitingWithdrawal, 0);
        let active = Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::Active, 0);
        let earn_awaiting = Delegation::mock_with(Chain::Ethereum, StakeProviderType::Earn, DelegationState::AwaitingWithdrawal, 0);

        assert_eq!(destination_kind(&delegation_destination(WalletType::Multicoin, asset.clone(), active)), "details");
        assert_eq!(destination_kind(&delegation_destination(WalletType::View, asset.clone(), awaiting.clone())), "details");
        assert_eq!(
            destination_kind(&delegation_destination(WalletType::Multicoin, Asset::from_chain(Chain::Ethereum), earn_awaiting)),
            "earn withdraw amount",
            "an earn position withdraws through the amount screen"
        );
        let transfer = confirm_transfer(delegation_destination(WalletType::Multicoin, asset, awaiting.clone()));
        assert_eq!(transfer.value, BigInt::from(awaiting.base.balance.clone()));
        assert_eq!(transfer.recipient.address, awaiting.validator.id);
    }

    #[test]
    fn test_each_delegation_action_leads_to_its_screen() {
        use GemDelegationAction::*;
        let validators = vec![DelegationValidator::mock_cosmos("other")];
        let destination = |chain, provider, action| {
            delegation_action_destination(
                Asset::from_chain(chain),
                Delegation::mock_with(chain, provider, DelegationState::Active, 0),
                action,
                validators.clone(),
            )
        };

        let GemDelegationDestination::Amount {
            asset,
            input: GemDelegationAmountInput::Stake {
                input: GemStakeAmountInput::Stake { validators: offered, validator },
            },
        } = destination(Chain::Ethereum, StakeProviderType::Stake, Stake)
        else {
            panic!("stake opens the amount screen")
        };
        assert_eq!(asset, Asset::from_chain(Chain::Ethereum));
        assert_eq!(offered, validators);
        assert_eq!(
            validator,
            Some(Delegation::mock_with(Chain::Ethereum, StakeProviderType::Stake, DelegationState::Active, 0).validator)
        );
        assert_eq!(destination_kind(&destination(Chain::Ethereum, StakeProviderType::Stake, Redelegate)), "redelegate amount");
        assert_eq!(
            destination_kind(&destination(Chain::Ethereum, StakeProviderType::Stake, Unstake)),
            "unstake amount",
            "a chain that unstakes a chosen amount asks for it"
        );
        assert_eq!(
            destination_kind(&destination(Chain::Solana, StakeProviderType::Stake, Unstake)),
            "confirm",
            "a chain that unstakes whole confirms at once"
        );
        assert_eq!(destination_kind(&destination(Chain::Solana, StakeProviderType::Stake, Withdraw)), "confirm");
        assert_eq!(destination_kind(&destination(Chain::Ethereum, StakeProviderType::Earn, Withdraw)), "earn withdraw amount");
        assert_eq!(destination_kind(&destination(Chain::Ethereum, StakeProviderType::Earn, Deposit)), "earn deposit amount");

        let unstaked = Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::Active, 0);
        let transfer = confirm_transfer(destination(Chain::Solana, StakeProviderType::Stake, Unstake));
        assert_eq!(transfer.value, BigInt::from(unstaked.base.balance.clone()));
        assert_eq!(transfer.recipient.address, unstaked.validator.id);
    }

    #[test]
    fn test_delegation_actions_follow_state_and_chain_config() {
        use GemDelegationAction::*;
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0)
            ),
            vec![Stake, Unstake, Redelegate]
        );
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Inactive, 0)
            ),
            vec![Unstake, Redelegate]
        );
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::Active, 0)
            ),
            vec![Unstake]
        );
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::AwaitingWithdrawal, 0)
            ),
            vec![Withdraw]
        );
        assert!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Pending, 0)
            )
            .is_empty()
        );
        assert!(
            delegation_actions(
                WalletType::View,
                &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0)
            )
            .is_empty()
        );
        assert!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Bitcoin, StakeProviderType::Stake, DelegationState::Active, 0)
            )
            .is_empty()
        );
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Ethereum, StakeProviderType::Earn, DelegationState::Active, 0)
            ),
            vec![Deposit, Withdraw]
        );
        assert_eq!(
            delegation_actions(
                WalletType::Multicoin,
                &Delegation::mock_with(Chain::Ethereum, StakeProviderType::Earn, DelegationState::Inactive, 0)
            ),
            vec![Withdraw]
        );
    }

    #[test]
    fn test_stake_actions_follow_the_wallet_chain_and_balance() {
        use GemStakeAction::*;
        let actions = |chain, has_validators, balance: GemAssetBalance, rewards: Vec<Delegation>| {
            stake_actions(WalletType::Multicoin, chain, has_validators, &balance, &rewards)
                .into_iter()
                .map(|item| (item.action, item.is_enabled, item.requires_frozen_balance))
                .collect::<Vec<_>>()
        };

        assert_eq!(actions(Chain::Cosmos, true, GemAssetBalance::mock(), vec![]), vec![(Stake, true, false)]);
        assert_eq!(actions(Chain::Cosmos, false, GemAssetBalance::mock(), vec![]), vec![(Stake, false, false)]);
        assert_eq!(
            actions(
                Chain::Cosmos,
                true,
                GemAssetBalance::mock(),
                vec![Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 5)]
            ),
            vec![(Stake, true, false), (ClaimRewards, true, false)]
        );
        assert_eq!(
            actions(Chain::Tron, true, GemAssetBalance::mock(), vec![]),
            vec![(Stake, true, true), (Freeze, true, false), (Unfreeze, true, false)],
            "a freeze chain with nothing frozen asks for a frozen balance before staking"
        );
        assert_eq!(
            actions(
                Chain::Tron,
                false,
                GemAssetBalance {
                    locked: BigUint::from(10u32),
                    ..GemAssetBalance::mock()
                },
                vec![]
            ),
            vec![(Stake, false, false), (Freeze, true, false), (Unfreeze, true, false)],
            "a locked balance counts as frozen"
        );
        assert!(
            stake_actions(
                WalletType::View,
                Chain::Cosmos,
                true,
                &GemAssetBalance::mock(),
                &[Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 5)]
            )
            .is_empty()
        );
        assert!(
            stake_actions(
                WalletType::Multicoin,
                Chain::Bitcoin,
                true,
                &GemAssetBalance::mock(),
                &[Delegation::mock_with(Chain::Bitcoin, StakeProviderType::Stake, DelegationState::Active, 5)]
            )
            .is_empty()
        );
    }

    #[test]
    fn test_can_claim_all_rewards() {
        assert!(can_claim_all_rewards(Chain::Cosmos, 3));
        assert!(!can_claim_all_rewards(Chain::Sui, 3));
        assert!(can_claim_all_rewards(Chain::Sui, 1));
        assert!(!can_claim_all_rewards(Chain::Bitcoin, 2));

        let one = claim_rewards(
            Chain::Sui,
            vec![
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 0),
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 7),
            ],
        );
        assert_eq!(one.value, BigInt::from(7));
        assert!(matches!(one.destination, GemClaimRewardsDestination::Transfer { ref transfer } if transfer.value == BigInt::from(7)));
        let several = claim_rewards(
            Chain::Sui,
            vec![
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 3),
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 4),
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 0),
            ],
        );
        assert_eq!(several.value, BigInt::from(7));
        assert!(matches!(several.destination, GemClaimRewardsDestination::Amount { ref delegations } if delegations.len() == 2));
        let cosmos = claim_rewards(
            Chain::Cosmos,
            vec![
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 3),
                Delegation::mock_with(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, 4),
            ],
        );
        assert!(matches!(cosmos.destination, GemClaimRewardsDestination::Transfer { .. }));
    }

    #[test]
    fn test_the_earn_rate_prefers_the_provider_that_would_take_the_deposit() {
        assert_eq!(
            earn_apr(
                &[
                    DelegationValidator {
                        apr: 4.5,
                        ..DelegationValidator::mock_cosmos("earn")
                    },
                    DelegationValidator {
                        apr: 9.9,
                        ..DelegationValidator::mock_cosmos("earn")
                    }
                ],
                Some(1.0)
            ),
            4.5,
            "the first provider is the one that would take the deposit"
        );
        assert_eq!(
            earn_apr(
                &[DelegationValidator {
                    apr: 0.0,
                    ..DelegationValidator::mock_cosmos("earn")
                }],
                Some(1.5)
            ),
            1.5,
            "a provider quoting nothing falls back to the asset's rate"
        );
        assert_eq!(earn_apr(&[], Some(2.5)), 2.5);
        assert_eq!(earn_apr(&[], None), 0.0);
    }

    #[test]
    fn test_can_claim_rewards() {
        assert!(can_claim_rewards(
            WalletType::Multicoin,
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 10)
        ));
        assert!(!can_claim_rewards(
            WalletType::Multicoin,
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0)
        ));
        assert!(!can_claim_rewards(
            WalletType::Multicoin,
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Inactive, 10)
        ));
        assert!(!can_claim_rewards(
            WalletType::View,
            &Delegation::mock_with(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 10)
        ));
        assert!(!can_claim_rewards(
            WalletType::Multicoin,
            &Delegation::mock_with(Chain::Solana, StakeProviderType::Stake, DelegationState::Active, 10)
        ));
    }

    #[test]
    fn test_recommended_validator_prefers_configured_ids() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        assert!(!recommended.is_empty());
        let validators = vec![DelegationValidator::mock_cosmos("other"), DelegationValidator::mock_cosmos(&recommended[0])];
        assert_eq!(
            recommended_validators(Chain::Cosmos, &validators)
                .iter()
                .map(|validator| validator.id.as_str())
                .collect::<Vec<_>>(),
            vec![recommended[0].as_str()]
        );
        assert_eq!(recommended_validator(Chain::Cosmos, validators).unwrap().id, recommended[0]);
        assert_eq!(recommended_validator(Chain::Cosmos, vec![DelegationValidator::mock_cosmos("other")]).unwrap().id, "other");
        assert!(recommended_validator(Chain::Cosmos, vec![]).is_none());
    }

    #[test]
    fn test_a_redelegate_never_lands_on_or_recommends_the_validator_it_leaves() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        let validators = vec![DelegationValidator::mock_cosmos("other"), DelegationValidator::mock_cosmos(&recommended[0])];
        let redelegate = |from: &str, validators: Vec<DelegationValidator>| {
            validator_selection(
                Chain::Cosmos,
                &GemStakeAmountInput::Redelegate {
                    validators,
                    delegation: Delegation::mock_with_validator(DelegationValidator::mock_cosmos(from)),
                    validator: None,
                },
            )
        };

        let leaving_other = redelegate("other", validators.clone());
        assert_eq!(leaving_other.validator.unwrap().validator.id, recommended[0]);
        assert_eq!(ids(&leaving_other.recommended), vec![recommended[0].as_str()]);

        let leaving_recommended = redelegate(&recommended[0], validators);
        assert_eq!(leaving_recommended.validator.unwrap().validator.id, "other");
        assert!(leaving_recommended.recommended.is_empty());

        assert!(redelegate("other", vec![DelegationValidator::mock_cosmos("other")]).validator.is_none());
    }

    fn ids(rows: &[GemValidatorRow]) -> Vec<&str> {
        rows.iter().map(|row| row.validator.id.as_str()).collect()
    }

    #[test]
    fn test_each_stake_action_picks_its_own_default_validator() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        let current = DelegationValidator::mock_cosmos("current");
        let validators = vec![current.clone(), DelegationValidator::mock_cosmos(&recommended[0])];
        let selection = |input| validator_selection(Chain::Cosmos, &input);

        let stake_fresh = selection(GemStakeAmountInput::Stake {
            validators: validators.clone(),
            validator: None,
        });
        assert_eq!(ids(&stake_fresh.recommended), vec![recommended[0].as_str()]);
        assert_eq!(stake_fresh.validator.unwrap().validator.id, recommended[0]);
        assert!(stake_fresh.can_select);

        let stake_more = selection(GemStakeAmountInput::Stake {
            validators: validators.clone(),
            validator: Some(current.clone()),
        });
        assert_eq!(stake_more.validator.unwrap().validator.id, "current");

        let redelegate = selection(GemStakeAmountInput::Redelegate {
            validators: validators.clone(),
            delegation: Delegation::mock_with_validator(current.clone()),
            validator: None,
        });
        assert_eq!(redelegate.validator.unwrap().validator.id, recommended[0]);
        assert!(redelegate.can_select);

        for held in [
            GemStakeAmountInput::Unstake {
                delegation: Delegation::mock_with_validator(current.clone()),
            },
            GemStakeAmountInput::Withdraw {
                delegation: Delegation::mock_with_validator(current.clone()),
            },
        ] {
            let held = selection(held);
            assert!(held.recommended.is_empty());
            assert_eq!(held.validator.unwrap().validator.id, "current");
            assert!(!held.can_select);
        }

        let one_reward = selection(GemStakeAmountInput::Rewards {
            delegations: vec![Delegation::mock_with_validator(current.clone())],
            validator: None,
        });
        assert_eq!(one_reward.validator.unwrap().validator.id, "current");
        assert!(!one_reward.can_select);

        let many_rewards = selection(GemStakeAmountInput::Rewards {
            delegations: vec![
                Delegation::mock_with_validator(current.clone()),
                Delegation::mock_with_validator(DelegationValidator::mock_cosmos(&recommended[0])),
            ],
            validator: None,
        });
        assert!(many_rewards.can_select);
        assert_eq!(many_rewards.validator.unwrap().validator.id, "current");

        let picked_reward = selection(GemStakeAmountInput::Rewards {
            delegations: vec![
                Delegation::mock_with_validator(current),
                Delegation::mock_with_validator(DelegationValidator::mock_cosmos(&recommended[0])),
            ],
            validator: Some(DelegationValidator::mock_cosmos(&recommended[0])),
        });
        assert_eq!(picked_reward.validator.unwrap().validator.id, recommended[0]);

        for resource in [
            GemStakeAmountInput::Freeze { resource: Resource::Bandwidth },
            GemStakeAmountInput::Unfreeze { resource: Resource::Energy },
        ] {
            assert_eq!(
                selection(resource),
                GemStakeValidatorSelection {
                    options: Vec::new(),
                    recommended: vec![],
                    validator: None,
                    can_select: false
                }
            );
        }
    }

    #[test]
    fn test_stake_type_needs_the_confirmed_validator_and_keeps_the_pick() {
        let current = DelegationValidator::mock_cosmos("current");
        let other = DelegationValidator::mock_cosmos("other");
        let validators = vec![current.clone(), other.clone()];

        let stake = GemStakeAmountInput::Stake {
            validators: validators.clone(),
            validator: None,
        };
        assert!(stake_type(&stake).is_err());
        assert!(matches!(stake_type(&with_validator(&stake, other.clone())).unwrap(), StakeType::Stake(validator) if validator.id == "other"));

        let redelegate = GemStakeAmountInput::Redelegate {
            validators,
            delegation: Delegation::mock_with_validator(current.clone()),
            validator: None,
        };
        assert!(stake_type(&redelegate).is_err());
        match stake_type(&with_validator(&redelegate, other.clone())).unwrap() {
            StakeType::Redelegate(data) => {
                assert_eq!(data.delegation.validator.id, "current");
                assert_eq!(data.to_validator.id, "other");
            }
            _ => panic!("expected a redelegate"),
        }

        let rewards = GemStakeAmountInput::Rewards {
            delegations: vec![Delegation::mock_with_validator(current.clone()), Delegation::mock_with_validator(other.clone())],
            validator: None,
        };
        assert!(stake_type(&rewards).is_err());
        assert!(matches!(stake_type(&with_validator(&rewards, other.clone())).unwrap(), StakeType::Rewards(validators) if validators.len() == 1 && validators[0].id == "other"));

        let unstake = GemStakeAmountInput::Unstake {
            delegation: Delegation::mock_with_validator(current.clone()),
        };
        assert!(matches!(stake_type(&with_validator(&unstake, other)).unwrap(), StakeType::Unstake(delegation) if delegation.validator.id == "current"));
        let withdraw = GemStakeAmountInput::Withdraw {
            delegation: Delegation::mock_with_validator(current),
        };
        assert!(matches!(stake_type(&withdraw).unwrap(), StakeType::Withdraw(delegation) if delegation.validator.id == "current"));

        let freeze = GemStakeAmountInput::Freeze { resource: Resource::Bandwidth };
        assert!(matches!(
            stake_type(&with_resource(&freeze, Resource::Energy)).unwrap(),
            StakeType::Freeze(Resource::Energy)
        ));
        let unfreeze = GemStakeAmountInput::Unfreeze { resource: Resource::Energy };
        assert!(matches!(
            stake_type(&with_resource(&unfreeze, Resource::Bandwidth)).unwrap(),
            StakeType::Unfreeze(Resource::Bandwidth)
        ));
        assert!(matches!(with_resource(&stake, Resource::Energy), GemStakeAmountInput::Stake { validator: None, .. }));
    }

    #[test]
    fn test_merge_validators_fills_names_and_keeps_active_first() {
        let names = HashMap::from([("b".to_string(), "Bee".to_string())]);
        let mut unnamed = DelegationValidator::mock_cosmos("b");
        unnamed.name = String::new();
        let mut inactive = DelegationValidator::mock_cosmos("a");
        inactive.is_active = false;

        let merged = merge_validators(
            vec![DelegationValidator::mock_cosmos("a"), unnamed],
            vec![inactive, DelegationValidator::mock_cosmos("c")],
            &names,
        );

        assert_eq!(merged.iter().map(|validator| validator.id.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert!(merged[0].is_active);
        assert_eq!(merged[1].name, "Bee");
    }

    #[test]
    fn test_missing_validators_become_inactive_placeholders() {
        let existing = HashMap::from([("known".to_string(), DelegationValidator::mock_cosmos("known"))]);
        let names = HashMap::from([("named".to_string(), "Named".to_string())]);

        let missing = missing_validators(
            Chain::Cosmos,
            &[
                DelegationBase::mock_with_validator("known"),
                DelegationBase::mock_with_validator("named"),
                DelegationBase::mock_with_validator("anon"),
                DelegationBase::mock_with_validator("anon"),
            ],
            &existing,
            &names,
        );

        assert_eq!(
            missing
                .iter()
                .map(|validator| (validator.id.as_str(), validator.name.as_str(), validator.is_active))
                .collect::<Vec<_>>(),
            vec![("named", "Named", false), ("anon", "anon", false)]
        );
        assert_eq!(
            stale_delegation_ids(
                vec![DelegationBase::mock_with_validator("known").id(), "gone".to_string()],
                &[DelegationBase::mock_with_validator("known")]
            ),
            vec!["gone"]
        );
        let applied = delegations_with_state(
            vec![DelegationBase::mock_with_validator("known"), DelegationBase::mock_with_validator("anon")],
            &HashMap::from([("anon".to_string(), inactive_validator(Chain::Cosmos, "anon".to_string(), String::new()))]),
        );
        assert_eq!(
            applied.iter().map(|delegation| delegation.state).collect::<Vec<_>>(),
            vec![DelegationState::Active, DelegationState::Inactive]
        );
    }

    #[test]
    fn test_validator_names_and_earn_apr() {
        let mut inactive = DelegationValidator::mock_cosmos("old");
        inactive.is_active = false;
        assert_eq!(
            stale_validator_ids(
                vec![DelegationValidator::mock_cosmos("kept"), DelegationValidator::mock_cosmos("gone"), inactive],
                &[DelegationValidator::mock_cosmos("kept")]
            ),
            vec!["gone"]
        );

        let names = validator_address_names(&[DelegationValidator::mock_cosmos("v1")]);
        assert_eq!(
            (names[0].address.as_str(), names[0].name.as_str(), &names[0].address_type),
            ("v1", "v1", &AddressType::Validator)
        );

        let earn = earn_validators(vec![DelegationValidator::mock_cosmos("p")], 4.5);
        assert_eq!(earn[0].apr, 4.5);
    }

    #[test]
    fn test_freeze_chains_need_a_frozen_balance_before_staking() {
        assert!(requires_frozen_balance(Chain::Tron, &BigUint::ZERO));
        assert!(!requires_frozen_balance(Chain::Tron, &BigUint::from(10u32)));
        assert!(!requires_frozen_balance(Chain::Cosmos, &BigUint::ZERO));
        assert!(!requires_frozen_balance(Chain::Bitcoin, &BigUint::ZERO));
    }

    #[test]
    fn test_claiming_needs_rewards_on_a_chain_that_claims() {
        assert!(can_claim_stake_rewards(Chain::Cosmos, &BigUint::from(10u32)));
        assert!(!can_claim_stake_rewards(Chain::Cosmos, &BigUint::ZERO));
        assert!(!can_claim_stake_rewards(Chain::Bitcoin, &BigUint::from(10u32)));
    }

    #[test]
    fn test_selectable_validators_drop_inactive_unnamed_and_system_entries_and_sort_by_apr() {
        let mut active = DelegationValidator::mock_cosmos("active");
        active.apr = 5.0;
        let mut best = DelegationValidator::mock_cosmos("best");
        best.apr = 9.0;
        let mut inactive = DelegationValidator::mock_cosmos("inactive");
        inactive.is_active = false;
        let mut unnamed = DelegationValidator::mock_cosmos("unnamed");
        unnamed.name = String::new();
        let system = DelegationValidator::mock_cosmos(DelegationValidator::SYSTEM_ID);
        let legacy_system = DelegationValidator::mock_cosmos("unstaking");

        let selectable = selectable_validators(vec![active, inactive, unnamed, system, legacy_system, best]);

        assert_eq!(selectable.iter().map(|validator| validator.id.as_str()).collect::<Vec<_>>(), vec!["best", "active"]);
    }

    #[test]
    fn test_staked_value_counts_rewards_on_delegating_chains() {
        assert_eq!(
            GemAssetBalance {
                staked: BigUint::from(100u32),
                pending: BigUint::from(20u32),
                rewards: BigUint::from(5u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Cosmos),
            GemBigUint::from(125u32)
        );
        assert_eq!(
            GemAssetBalance {
                staked: BigUint::from(100u32),
                pending: BigUint::from(20u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Cosmos),
            GemBigUint::from(120u32)
        );
        assert_eq!(
            GemAssetBalance {
                staked: BigUint::from(700u32),
                pending: BigUint::from(30u32),
                rewards: BigUint::from(3u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Solana),
            GemBigUint::from(733u32)
        );
        assert_eq!(
            GemAssetBalance {
                frozen: BigUint::from(9u32),
                locked: BigUint::from(9u32),
                staked: BigUint::from(100u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Cosmos),
            GemBigUint::from(100u32)
        );
    }

    #[test]
    fn test_unclaimed_rewards_alone_are_a_staked_position() {
        let rewards_only = GemAssetBalance {
            rewards: BigUint::from(7u32),
            ..GemAssetBalance::mock()
        };
        assert_eq!(rewards_only.staked_value(Chain::Cosmos), GemBigUint::from(7u32));
        assert!(rewards_only.shows_stake_balance(Chain::Cosmos, false));
    }

    #[test]
    fn test_staked_value_uses_the_frozen_balance_on_freeze_chains() {
        assert_eq!(
            GemAssetBalance {
                frozen: BigUint::from(40u32),
                locked: BigUint::from(60u32),
                pending: BigUint::from(10u32),
                rewards: BigUint::from(5u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Tron),
            GemBigUint::from(115u32)
        );
        assert_eq!(
            GemAssetBalance {
                frozen: BigUint::from(40u32),
                locked: BigUint::from(60u32),
                staked: BigUint::from(999u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Tron),
            GemBigUint::from(100u32)
        );
    }

    #[test]
    fn test_shows_stake_balance_when_enabled_or_holding_a_position() {
        assert!(GemAssetBalance::mock().shows_stake_balance(Chain::Cosmos, true));
        assert!(!GemAssetBalance::mock().shows_stake_balance(Chain::Cosmos, false));
        assert!(
            GemAssetBalance {
                rewards: BigUint::from(5u32),
                ..GemAssetBalance::mock()
            }
            .shows_stake_balance(Chain::Cosmos, false)
        );
        assert!(
            GemAssetBalance {
                frozen: BigUint::from(40u32),
                ..GemAssetBalance::mock()
            }
            .shows_stake_balance(Chain::Tron, false)
        );
        assert!(
            !GemAssetBalance {
                staked: BigUint::from(40u32),
                ..GemAssetBalance::mock()
            }
            .shows_stake_balance(Chain::Tron, false)
        );
        assert!(!GemAssetBalance::mock().shows_stake_balance(Chain::Bitcoin, true));
    }

    #[test]
    fn test_detail_rows_list_what_the_wallet_holds_beyond_available() {
        use GemBalanceRow::*;
        let nothing = GemAssetBalance {
            available: BigUint::from(5u32),
            ..GemAssetBalance::mock()
        };
        assert!(nothing.detail_rows(Chain::Ethereum, false).is_empty(), "only an available balance needs no breakdown");
        assert_eq!(
            nothing.detail_rows(Chain::Cosmos, true),
            vec![Staked { value: BigUint::ZERO }],
            "a stakeable chain offers staking before anything is staked"
        );

        let staked = GemAssetBalance {
            available: BigUint::from(5u32),
            staked: BigUint::from(100u32),
            rewards: BigUint::from(1u32),
            reserved: BigUint::from(2u32),
            pending_unconfirmed: BigUint::from(3u32),
            ..GemAssetBalance::mock()
        };
        assert_eq!(
            staked.detail_rows(Chain::Cosmos, false),
            vec![
                Available { value: BigUint::from(5u32) },
                Staked { value: BigUint::from(101u32) },
                PendingUnconfirmed { value: BigUint::from(3u32) },
                Reserved {
                    value: BigUint::from(2u32),
                    url: None
                },
            ]
        );
        let reserved = GemAssetBalance {
            reserved: BigUint::from(2u32),
            ..GemAssetBalance::mock()
        };
        assert_eq!(
            reserved.detail_rows(Chain::Xrp, false).last(),
            Some(&Reserved {
                value: BigUint::from(2u32),
                url: account_activation_fee_url(Chain::Xrp)
            })
        );
        let earn = GemAssetBalance {
            earn: BigUint::from(7u32),
            ..GemAssetBalance::mock()
        };
        assert_eq!(earn.detail_rows(Chain::Ethereum, false).contains(&Earn { value: BigUint::from(7u32) }), EARN_OFFERED);
    }

    #[test]
    fn test_stake_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemAssetBalance) -> (GemBigUint, GemBigUint, GemBigUint, GemBigUint, GemBigUint) =
            |balance| (balance.frozen, balance.locked, balance.staked, balance.pending, balance.rewards);
        assert_eq!(
            GemAssetBalance {
                staked: BigUint::from(100u32),
                pending: BigUint::from(20u32),
                rewards: BigUint::from(5u32),
                ..GemAssetBalance::mock()
            }
            .staked_value(Chain::Cosmos),
            GemBigUint::from(125u32)
        );
    }

    #[test]
    fn delegations_sort_by_balance_descending() {
        let mut small = Delegation::mock();
        small.base.balance = BigUint::from(10u64);
        let mut large = Delegation::mock();
        large.base.balance = BigUint::from(300u64);
        assert_eq!(sorted_delegations(vec![small.clone(), large.clone()]), vec![large, small]);
    }
}
