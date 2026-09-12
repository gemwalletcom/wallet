use std::collections::{HashMap, HashSet};

use crate::services::collections::{stale, unique};

use num_bigint::{BigInt, BigUint};
use primitives::AddressName;
use primitives::{
    AddressFormatStyle, AddressFormatter, AddressType, Asset, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, RedelegateData, Resource, StakeChain,
    StakeProviderType, StakeType, VerificationStatus, WalletType, YieldProvider,
};
use rand::seq::IndexedRandom;
use std::str::FromStr;

use super::model::{
    GemClaimRewards, GemClaimRewardsDestination, GemDelegationAction, GemDelegationCompletion, GemDelegationDestination, GemDelegationStatus, GemDelegationTone, GemStakeAction,
    GemStakeActionItem, GemStakeAmountInput, GemStakeValidatorSelection, GemValidatorRow,
};
use crate::config::image::GemImage;
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
    let value = BigInt::from(delegation.base.balance.clone());
    GemDelegationDestination::Withdraw {
        transfer: transfer_rules::stake_transfer_data(asset, StakeType::Withdraw(delegation), value, false),
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
    providers
        .first()
        .map(|provider| provider.apr)
        .filter(|apr| *apr > 0.0)
        .or(asset_apr)
        .unwrap_or_default()
}

pub fn can_claim_rewards(wallet_type: WalletType, delegation: &Delegation) -> bool {
    let Some(config) = stake_config(delegation.base.asset_id.chain) else {
        return false;
    };
    wallet_type != WalletType::View && config.can_claim_rewards && shows_rewards(&delegation.base)
}

pub fn validator_display_name(validator: &DelegationValidator) -> String {
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

pub fn lock_time_seconds(chain: Chain) -> u64 {
    stake_config(chain).map(|config| config.time_lock).unwrap_or_default()
}

pub fn min_stake_amount(chain: Chain) -> BigInt {
    stake_config(chain).map(|config| BigInt::from(config.min_amount)).unwrap_or_default()
}

pub fn can_change_amount_on_unstake(chain: Chain) -> bool {
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
            positive(&self.earn).map(|value| GemBalanceRow::Earn { value }),
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
    pub fn shows_stake_balance(&self, chain: Chain, is_stake_enabled: bool) -> bool {
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

pub fn recommended_validators(chain: Chain, validators: &[DelegationValidator]) -> Vec<DelegationValidator> {
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

pub fn apply_validator_state(delegations: Vec<DelegationBase>, validators: &HashMap<String, DelegationValidator>) -> Vec<DelegationBase> {
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
    use super::*;
    use primitives::{AssetId, Resource};

    fn solana_validator(name: &str) -> DelegationValidator {
        DelegationValidator {
            chain: Chain::Solana,
            id: "8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD".to_string(),
            name: name.to_string(),
            ..DelegationValidator::mock()
        }
    }

    #[test]
    fn test_validator_display_name() {
        assert_eq!(validator_display_name(&DelegationValidator::mock()), "Test Validator");
        assert_eq!(validator_display_name(&solana_validator("")), "8GbwA...JF8iD");

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
        let row = validator_row(&solana_validator(""));
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
        let named = solana_validator("Everstake");

        let names = validator_address_names(&[named.clone(), solana_validator("")]);

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

    fn stake_balance(frozen: u32, locked: u32, staked: u32, pending: u32, rewards: u32) -> GemAssetBalance {
        GemAssetBalance {
            frozen: BigUint::from(frozen),
            locked: BigUint::from(locked),
            staked: BigUint::from(staked),
            pending: BigUint::from(pending),
            rewards: BigUint::from(rewards),
            ..GemAssetBalance::mock()
        }
    }

    fn validator(id: &str) -> DelegationValidator {
        DelegationValidator {
            chain: Chain::Cosmos,
            id: id.to_string(),
            name: id.to_string(),
            is_active: true,
            commission: 0.0,
            apr: 1.0,
            provider_type: StakeProviderType::Stake,
        }
    }

    #[test]
    fn test_validator_explorer_address_skips_system_and_earn_validators() {
        assert_eq!(validator_explorer_address(&validator("cosmosvaloper1")), Some("cosmosvaloper1".to_string()));
        assert_eq!(validator_explorer_address(&validator(DelegationValidator::SYSTEM_ID)), None);
        assert_eq!(validator_explorer_address(&validator("unstaking")), None);
        assert_eq!(
            validator_explorer_address(&DelegationValidator {
                provider_type: StakeProviderType::Earn,
                ..validator("cosmosvaloper1")
            }),
            None
        );
    }

    fn delegation(chain: Chain, provider: StakeProviderType, state: DelegationState, rewards: u32) -> Delegation {
        let validator = DelegationValidator::mock();
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(chain),
                state,
                rewards: BigUint::from(rewards),
                ..DelegationBase::mock()
            },
            validator: DelegationValidator {
                chain,
                provider_type: provider,
                ..validator
            },
        }
    }

    #[test]
    fn test_delegation_status_presents_the_state_its_tone_and_the_completion_label() {
        let status = |state, provider_type| delegation_status(&delegation(Chain::Cosmos, provider_type, state, 100));

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

        let mut on_inactive_validator = delegation(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 100);
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
            let base = delegation(Chain::Cosmos, StakeProviderType::Stake, state, 100).base;
            assert!(!shows_rewards(&base));
        }
        assert!(shows_rewards(&delegation(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 100).base));
        assert!(!shows_rewards(&delegation(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active, 0).base));
        assert!(!shows_rewards(&delegation(Chain::Cosmos, StakeProviderType::Stake, DelegationState::Inactive, 100).base));
    }

    #[test]
    fn test_a_delegation_tap_withdraws_only_an_awaiting_withdrawal_on_a_signing_wallet() {
        let asset = Asset::from_chain(Chain::Solana);
        let awaiting = delegation(Chain::Solana, StakeProviderType::Stake, DelegationState::AwaitingWithdrawal, 0);
        let active = delegation(Chain::Solana, StakeProviderType::Stake, DelegationState::Active, 0);

        assert!(matches!(
            delegation_destination(WalletType::Multicoin, asset.clone(), active),
            GemDelegationDestination::Details
        ));
        assert!(matches!(
            delegation_destination(WalletType::View, asset.clone(), awaiting.clone()),
            GemDelegationDestination::Details
        ));
        let GemDelegationDestination::Withdraw { transfer } = delegation_destination(WalletType::Multicoin, asset.clone(), awaiting.clone()) else {
            panic!("expected a withdraw transfer");
        };
        assert_eq!(transfer.value, BigInt::from(awaiting.base.balance.clone()));
        assert_eq!(transfer.recipient.address, awaiting.validator.id);
    }

    #[test]
    fn test_delegation_actions_follow_state_and_chain_config() {
        use GemDelegationAction::*;
        let stake = |chain, state| delegation(chain, StakeProviderType::Stake, state, 0);
        let earn = |chain, state| delegation(chain, StakeProviderType::Earn, state, 0);
        assert_eq!(
            delegation_actions(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Active)),
            vec![Stake, Unstake, Redelegate]
        );
        assert_eq!(
            delegation_actions(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Inactive)),
            vec![Unstake, Redelegate]
        );
        assert_eq!(delegation_actions(WalletType::Multicoin, &stake(Chain::Solana, DelegationState::Active)), vec![Unstake]);
        assert_eq!(
            delegation_actions(WalletType::Multicoin, &stake(Chain::Solana, DelegationState::AwaitingWithdrawal)),
            vec![Withdraw]
        );
        assert!(delegation_actions(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Pending)).is_empty());
        assert!(delegation_actions(WalletType::View, &stake(Chain::Cosmos, DelegationState::Active)).is_empty());
        assert!(delegation_actions(WalletType::Multicoin, &stake(Chain::Bitcoin, DelegationState::Active)).is_empty());
        assert_eq!(
            delegation_actions(WalletType::Multicoin, &earn(Chain::Ethereum, DelegationState::Active)),
            vec![Deposit, Withdraw]
        );
        assert_eq!(delegation_actions(WalletType::Multicoin, &earn(Chain::Ethereum, DelegationState::Inactive)), vec![Withdraw]);
    }

    #[test]
    fn test_stake_actions_follow_the_wallet_chain_and_balance() {
        use GemStakeAction::*;
        let balance = |frozen: u32, locked: u32| stake_balance(frozen, locked, 0, 0, 0);
        let rewards = |chain, rewards| vec![delegation(chain, StakeProviderType::Stake, DelegationState::Active, rewards)];
        let actions = |chain, has_validators, balance: GemAssetBalance, rewards: Vec<Delegation>| {
            stake_actions(WalletType::Multicoin, chain, has_validators, &balance, &rewards)
                .into_iter()
                .map(|item| (item.action, item.is_enabled, item.requires_frozen_balance))
                .collect::<Vec<_>>()
        };

        assert_eq!(actions(Chain::Cosmos, true, balance(0, 0), vec![]), vec![(Stake, true, false)]);
        assert_eq!(actions(Chain::Cosmos, false, balance(0, 0), vec![]), vec![(Stake, false, false)]);
        assert_eq!(
            actions(Chain::Cosmos, true, balance(0, 0), rewards(Chain::Cosmos, 5)),
            vec![(Stake, true, false), (ClaimRewards, true, false)]
        );
        assert_eq!(
            actions(Chain::Tron, true, balance(0, 0), vec![]),
            vec![(Stake, true, true), (Freeze, true, false), (Unfreeze, true, false)],
            "a freeze chain with nothing frozen asks for a frozen balance before staking"
        );
        assert_eq!(
            actions(Chain::Tron, false, balance(0, 10), vec![]),
            vec![(Stake, false, false), (Freeze, true, false), (Unfreeze, true, false)],
            "a locked balance counts as frozen"
        );
        assert!(stake_actions(WalletType::View, Chain::Cosmos, true, &balance(0, 0), &rewards(Chain::Cosmos, 5)).is_empty());
        assert!(stake_actions(WalletType::Multicoin, Chain::Bitcoin, true, &balance(0, 0), &rewards(Chain::Bitcoin, 5)).is_empty());
    }

    #[test]
    fn test_can_claim_all_rewards() {
        assert!(can_claim_all_rewards(Chain::Cosmos, 3));
        assert!(!can_claim_all_rewards(Chain::Sui, 3));
        assert!(can_claim_all_rewards(Chain::Sui, 1));
        assert!(!can_claim_all_rewards(Chain::Bitcoin, 2));

        let sui = |rewards| delegation(Chain::Sui, StakeProviderType::Stake, DelegationState::Active, rewards);
        let one = claim_rewards(Chain::Sui, vec![sui(0), sui(7)]);
        assert_eq!(one.value, BigInt::from(7));
        assert!(matches!(one.destination, GemClaimRewardsDestination::Transfer { ref transfer } if transfer.value == BigInt::from(7)));
        let several = claim_rewards(Chain::Sui, vec![sui(3), sui(4), sui(0)]);
        assert_eq!(several.value, BigInt::from(7));
        assert!(matches!(several.destination, GemClaimRewardsDestination::Amount { ref delegations } if delegations.len() == 2));
        let cosmos = claim_rewards(Chain::Cosmos, vec![sui(3), sui(4)]);
        assert!(matches!(cosmos.destination, GemClaimRewardsDestination::Transfer { .. }));
    }

    #[test]
    fn test_the_earn_rate_prefers_the_provider_that_would_take_the_deposit() {
        let provider = |apr: f64| DelegationValidator { apr, ..validator("earn") };

        assert_eq!(earn_apr(&[provider(4.5), provider(9.9)], Some(1.0)), 4.5, "the first provider is the one that would take the deposit");
        assert_eq!(earn_apr(&[provider(0.0)], Some(1.5)), 1.5, "a provider quoting nothing falls back to the asset's rate");
        assert_eq!(earn_apr(&[], Some(2.5)), 2.5);
        assert_eq!(earn_apr(&[], None), 0.0);
    }

    #[test]
    fn test_can_claim_rewards() {
        let stake = |chain, state, rewards| delegation(chain, StakeProviderType::Stake, state, rewards);
        assert!(can_claim_rewards(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Active, 10)));
        assert!(!can_claim_rewards(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Active, 0)));
        assert!(!can_claim_rewards(WalletType::Multicoin, &stake(Chain::Cosmos, DelegationState::Inactive, 10)));
        assert!(!can_claim_rewards(WalletType::View, &stake(Chain::Cosmos, DelegationState::Active, 10)));
        assert!(!can_claim_rewards(WalletType::Multicoin, &stake(Chain::Solana, DelegationState::Active, 10)));
    }

    #[test]
    fn test_recommended_validator_prefers_configured_ids() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        assert!(!recommended.is_empty());
        let validators = vec![validator("other"), validator(&recommended[0])];
        assert_eq!(
            recommended_validators(Chain::Cosmos, &validators)
                .iter()
                .map(|validator| validator.id.as_str())
                .collect::<Vec<_>>(),
            vec![recommended[0].as_str()]
        );
        assert_eq!(recommended_validator(Chain::Cosmos, validators).unwrap().id, recommended[0]);
        assert_eq!(recommended_validator(Chain::Cosmos, vec![validator("other")]).unwrap().id, "other");
        assert!(recommended_validator(Chain::Cosmos, vec![]).is_none());
    }

    #[test]
    fn test_a_redelegate_never_lands_on_or_recommends_the_validator_it_leaves() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        let validators = vec![validator("other"), validator(&recommended[0])];
        let redelegate = |from: &str, validators: Vec<DelegationValidator>| {
            validator_selection(
                Chain::Cosmos,
                &GemStakeAmountInput::Redelegate {
                    validators,
                    delegation: delegation_to(validator(from)),
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

        assert!(redelegate("other", vec![validator("other")]).validator.is_none());
    }

    fn ids(rows: &[GemValidatorRow]) -> Vec<&str> {
        rows.iter().map(|row| row.validator.id.as_str()).collect()
    }

    fn delegation_to(validator: DelegationValidator) -> Delegation {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Cosmos),
                validator_id: validator.id.clone(),
                ..DelegationBase::mock()
            },
            validator,
        }
    }

    #[test]
    fn test_each_stake_action_picks_its_own_default_validator() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        let current = validator("current");
        let validators = vec![current.clone(), validator(&recommended[0])];
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
            delegation: delegation_to(current.clone()),
            validator: None,
        });
        assert_eq!(redelegate.validator.unwrap().validator.id, recommended[0]);
        assert!(redelegate.can_select);

        for held in [
            GemStakeAmountInput::Unstake {
                delegation: delegation_to(current.clone()),
            },
            GemStakeAmountInput::Withdraw {
                delegation: delegation_to(current.clone()),
            },
        ] {
            let held = selection(held);
            assert!(held.recommended.is_empty());
            assert_eq!(held.validator.unwrap().validator.id, "current");
            assert!(!held.can_select);
        }

        let one_reward = selection(GemStakeAmountInput::Rewards {
            delegations: vec![delegation_to(current.clone())],
            validator: None,
        });
        assert_eq!(one_reward.validator.unwrap().validator.id, "current");
        assert!(!one_reward.can_select);

        let many_rewards = selection(GemStakeAmountInput::Rewards {
            delegations: vec![delegation_to(current.clone()), delegation_to(validator(&recommended[0]))],
            validator: None,
        });
        assert!(many_rewards.can_select);
        assert_eq!(many_rewards.validator.unwrap().validator.id, "current");

        let picked_reward = selection(GemStakeAmountInput::Rewards {
            delegations: vec![delegation_to(current), delegation_to(validator(&recommended[0]))],
            validator: Some(validator(&recommended[0])),
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
        let current = validator("current");
        let other = validator("other");
        let validators = vec![current.clone(), other.clone()];

        let stake = GemStakeAmountInput::Stake {
            validators: validators.clone(),
            validator: None,
        };
        assert!(stake_type(&stake).is_err());
        assert!(matches!(stake_type(&with_validator(&stake, other.clone())).unwrap(), StakeType::Stake(validator) if validator.id == "other"));

        let redelegate = GemStakeAmountInput::Redelegate {
            validators,
            delegation: delegation_to(current.clone()),
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
            delegations: vec![delegation_to(current.clone()), delegation_to(other.clone())],
            validator: None,
        };
        assert!(stake_type(&rewards).is_err());
        assert!(matches!(stake_type(&with_validator(&rewards, other.clone())).unwrap(), StakeType::Rewards(validators) if validators.len() == 1 && validators[0].id == "other"));

        let unstake = GemStakeAmountInput::Unstake {
            delegation: delegation_to(current.clone()),
        };
        assert!(matches!(stake_type(&with_validator(&unstake, other)).unwrap(), StakeType::Unstake(delegation) if delegation.validator.id == "current"));
        let withdraw = GemStakeAmountInput::Withdraw {
            delegation: delegation_to(current),
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
        let mut unnamed = validator("b");
        unnamed.name = String::new();
        let mut inactive = validator("a");
        inactive.is_active = false;

        let merged = merge_validators(vec![validator("a"), unnamed], vec![inactive, validator("c")], &names);

        assert_eq!(merged.iter().map(|validator| validator.id.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert!(merged[0].is_active);
        assert_eq!(merged[1].name, "Bee");
    }

    #[test]
    fn test_missing_validators_become_inactive_placeholders() {
        let delegation = |validator_id: &str| DelegationBase {
            asset_id: AssetId::from_chain(Chain::Cosmos),
            state: DelegationState::Active,
            balance: BigUint::from(1u8),
            shares: BigUint::ZERO,
            rewards: BigUint::ZERO,
            completion_date: None,
            delegation_id: validator_id.to_string(),
            validator_id: validator_id.to_string(),
        };
        let existing = HashMap::from([("known".to_string(), validator("known"))]);
        let names = HashMap::from([("named".to_string(), "Named".to_string())]);

        let missing = missing_validators(
            Chain::Cosmos,
            &[delegation("known"), delegation("named"), delegation("anon"), delegation("anon")],
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
            stale_delegation_ids(vec![delegation("known").id(), "gone".to_string()], &[delegation("known")]),
            vec!["gone"]
        );
        let applied = apply_validator_state(
            vec![delegation("known"), delegation("anon")],
            &HashMap::from([("anon".to_string(), inactive_validator(Chain::Cosmos, "anon".to_string(), String::new()))]),
        );
        assert_eq!(
            applied.iter().map(|delegation| delegation.state).collect::<Vec<_>>(),
            vec![DelegationState::Active, DelegationState::Inactive]
        );
    }

    #[test]
    fn test_validator_names_and_earn_apr() {
        let mut inactive = validator("old");
        inactive.is_active = false;
        assert_eq!(
            stale_validator_ids(vec![validator("kept"), validator("gone"), inactive], &[validator("kept")]),
            vec!["gone"]
        );

        let names = validator_address_names(&[validator("v1")]);
        assert_eq!(
            (names[0].address.as_str(), names[0].name.as_str(), &names[0].address_type),
            ("v1", "v1", &AddressType::Validator)
        );

        let earn = earn_validators(vec![validator("p")], 4.5);
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
        let mut active = validator("active");
        active.apr = 5.0;
        let mut best = validator("best");
        best.apr = 9.0;
        let mut inactive = validator("inactive");
        inactive.is_active = false;
        let mut unnamed = validator("unnamed");
        unnamed.name = String::new();
        let system = validator(DelegationValidator::SYSTEM_ID);
        let legacy_system = validator("unstaking");

        let selectable = selectable_validators(vec![active, inactive, unnamed, system, legacy_system, best]);

        assert_eq!(selectable.iter().map(|validator| validator.id.as_str()).collect::<Vec<_>>(), vec!["best", "active"]);
    }

    #[test]
    fn test_staked_value_counts_rewards_on_delegating_chains() {
        assert_eq!(stake_balance(0, 0, 100, 20, 5).staked_value(Chain::Cosmos), GemBigUint::from(125u32));
        assert_eq!(stake_balance(0, 0, 100, 20, 0).staked_value(Chain::Cosmos), GemBigUint::from(120u32));
        assert_eq!(stake_balance(0, 0, 700, 30, 3).staked_value(Chain::Solana), GemBigUint::from(733u32));
        assert_eq!(stake_balance(9, 9, 100, 0, 0).staked_value(Chain::Cosmos), GemBigUint::from(100u32));
    }

    #[test]
    fn test_unclaimed_rewards_alone_are_a_staked_position() {
        let rewards_only = stake_balance(0, 0, 0, 0, 7);
        assert_eq!(rewards_only.staked_value(Chain::Cosmos), GemBigUint::from(7u32));
        assert!(rewards_only.shows_stake_balance(Chain::Cosmos, false));
    }

    #[test]
    fn test_staked_value_uses_the_frozen_balance_on_freeze_chains() {
        assert_eq!(stake_balance(40, 60, 0, 10, 5).staked_value(Chain::Tron), GemBigUint::from(115u32));
        assert_eq!(stake_balance(40, 60, 999, 0, 0).staked_value(Chain::Tron), GemBigUint::from(100u32));
    }

    #[test]
    fn test_shows_stake_balance_when_enabled_or_holding_a_position() {
        assert!(stake_balance(0, 0, 0, 0, 0).shows_stake_balance(Chain::Cosmos, true));
        assert!(!stake_balance(0, 0, 0, 0, 0).shows_stake_balance(Chain::Cosmos, false));
        assert!(stake_balance(0, 0, 0, 0, 5).shows_stake_balance(Chain::Cosmos, false));
        assert!(stake_balance(40, 0, 0, 0, 0).shows_stake_balance(Chain::Tron, false));
        assert!(!stake_balance(0, 0, 40, 0, 0).shows_stake_balance(Chain::Tron, false));
        assert!(!stake_balance(0, 0, 0, 0, 0).shows_stake_balance(Chain::Bitcoin, true));
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
        assert_eq!(
            earn.detail_rows(Chain::Ethereum, false),
            vec![Available { value: BigUint::ZERO }, Earn { value: BigUint::from(7u32) }]
        );
    }

    #[test]
    fn test_stake_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemAssetBalance) -> (GemBigUint, GemBigUint, GemBigUint, GemBigUint, GemBigUint) =
            |balance| (balance.frozen, balance.locked, balance.staked, balance.pending, balance.rewards);
        assert_eq!(stake_balance(0, 0, 100, 20, 5).staked_value(Chain::Cosmos), GemBigUint::from(125u32));
    }
}
