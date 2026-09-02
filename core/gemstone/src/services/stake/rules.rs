use std::collections::{HashMap, HashSet};

use crate::services::collections::{stale, unique};

use std::str::FromStr;

use num_bigint::BigUint;
use primitives::AddressName;
use primitives::{AddressType, Chain, DelegationBase, DelegationState, DelegationValidator, StakeChain, StakeProviderType, VerificationStatus, WalletType};
use rand::seq::IndexedRandom;

use super::model::{GemDelegationAction, GemStakeAction, GemStakeActionItem, GemStakeBalance};
use crate::models::custom_types::GemBigInt;

const SYSTEM_VALIDATOR_IDS: [&str; 2] = [DelegationValidator::SYSTEM_ID, "unstaking"];
use crate::config::stake::get_stake_config;
use crate::config::validators::get_validators;

pub fn delegation_actions(wallet_type: WalletType, chain: Chain, provider: StakeProviderType, state: DelegationState) -> Vec<GemDelegationAction> {
    if wallet_type == WalletType::View {
        return vec![];
    }
    match provider {
        StakeProviderType::Stake => {
            let Some(config) = StakeChain::from_str(chain.as_ref()).ok().map(get_stake_config) else {
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

pub fn can_claim_rewards(wallet_type: WalletType, chain: Chain, state: DelegationState, rewards: &str) -> bool {
    let Some(config) = StakeChain::from_str(chain.as_ref()).ok().map(get_stake_config) else {
        return false;
    };
    wallet_type != WalletType::View && config.can_claim_rewards && state == DelegationState::Active && BigUint::from_str(rewards).is_ok_and(|rewards| rewards > BigUint::ZERO)
}

pub fn validator_explorer_address(validator: &DelegationValidator) -> Option<String> {
    match validator.provider_type {
        StakeProviderType::Stake if !SYSTEM_VALIDATOR_IDS.contains(&validator.id.as_str()) => Some(validator.id.clone()),
        StakeProviderType::Stake | StakeProviderType::Earn => None,
    }
}

pub fn shows_completion_date(state: DelegationState) -> bool {
    match state {
        DelegationState::Pending | DelegationState::Activating | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal => true,
        DelegationState::Active | DelegationState::Inactive => false,
    }
}

pub fn shows_rewards(state: DelegationState, rewards: &str) -> bool {
    state == DelegationState::Active && is_positive(rewards)
}

pub fn requires_frozen_balance(chain: Chain, frozen_value: &str) -> bool {
    let Some(config) = StakeChain::from_str(chain.as_ref()).ok().map(get_stake_config) else {
        return false;
    };
    config.uses_freeze && !is_positive(frozen_value)
}

pub fn can_claim_stake_rewards(chain: Chain, rewards_value: &str) -> bool {
    let Some(config) = StakeChain::from_str(chain.as_ref()).ok().map(get_stake_config) else {
        return false;
    };
    config.can_claim_rewards && is_positive(rewards_value)
}

pub fn can_claim_all_rewards(chain: Chain, delegations_with_rewards: u32) -> bool {
    let claims_all = StakeChain::from_str(chain.as_ref())
        .ok()
        .map(get_stake_config)
        .is_some_and(|config| config.can_claim_all_rewards);
    claims_all || delegations_with_rewards == 1
}

pub fn stake_actions(wallet_type: WalletType, chain: Chain, has_validators: bool, frozen_value: &str, rewards_value: &str) -> Vec<GemStakeActionItem> {
    let Some(config) = StakeChain::from_str(chain.as_ref()).ok().map(get_stake_config).filter(|_| wallet_type != WalletType::View) else {
        return vec![];
    };
    let uses_freeze = config.uses_freeze;
    let requires_frozen_balance = requires_frozen_balance(chain, frozen_value);
    let item = |action: GemStakeAction, is_enabled: bool, requires_frozen_balance: bool| GemStakeActionItem {
        action,
        is_enabled,
        requires_frozen_balance,
    };
    [
        Some(item(GemStakeAction::Stake, has_validators || requires_frozen_balance, requires_frozen_balance)),
        uses_freeze.then(|| item(GemStakeAction::Freeze, true, false)),
        uses_freeze.then(|| item(GemStakeAction::Unfreeze, true, false)),
        can_claim_stake_rewards(chain, rewards_value).then(|| item(GemStakeAction::ClaimRewards, true, false)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn is_positive(amount: &str) -> bool {
    BigUint::from_str(amount).is_ok_and(|amount| amount > BigUint::ZERO)
}

#[uniffi::export]
impl GemStakeBalance {
    pub fn staked_value(&self, chain: Chain) -> GemBigInt {
        let principal = match StakeChain::from_str(chain.as_ref()) {
            Ok(stake_chain) if stake_chain.get_uses_freeze() => &self.frozen + &self.locked,
            _ => self.staked.clone(),
        };
        principal + &self.pending + &self.rewards
    }

    pub fn shows_stake_balance(&self, chain: Chain, is_stake_enabled: bool) -> bool {
        StakeChain::from_str(chain.as_ref()).is_ok() && (is_stake_enabled || self.staked_value(chain) > GemBigInt::ZERO)
    }
}

pub fn selectable_validators(validators: Vec<DelegationValidator>) -> Vec<DelegationValidator> {
    let mut selectable: Vec<DelegationValidator> = validators
        .into_iter()
        .filter(|validator| validator.is_active && !validator.name.trim().is_empty() && !SYSTEM_VALIDATOR_IDS.contains(&validator.id.as_str()))
        .collect();
    selectable.sort_by(|left, right| right.apr.total_cmp(&left.apr));
    selectable
}

pub fn recommended_validator_ids(chain: Chain) -> Vec<String> {
    get_validators().remove(chain.as_ref()).unwrap_or_default()
}

pub fn recommended_validator(chain: Chain, validators: Vec<DelegationValidator>) -> Option<DelegationValidator> {
    let recommended = recommended_validator_ids(chain);
    let candidates: Vec<&DelegationValidator> = validators.iter().filter(|validator| recommended.contains(&validator.id)).collect();
    candidates
        .choose(&mut rand::rng())
        .map(|validator| (*validator).clone())
        .or_else(|| validators.first().cloned())
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
    use primitives::AssetId;

    fn stake_balance(frozen: i64, locked: i64, staked: i64, pending: i64, rewards: i64) -> GemStakeBalance {
        GemStakeBalance {
            frozen: GemBigInt::from(frozen),
            locked: GemBigInt::from(locked),
            staked: GemBigInt::from(staked),
            pending: GemBigInt::from(pending),
            rewards: GemBigInt::from(rewards),
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

    #[test]
    fn test_delegation_rows_follow_state() {
        for state in [
            DelegationState::Pending,
            DelegationState::Activating,
            DelegationState::Deactivating,
            DelegationState::AwaitingWithdrawal,
        ] {
            assert!(shows_completion_date(state));
            assert!(!shows_rewards(state, "100"));
        }
        for state in [DelegationState::Active, DelegationState::Inactive] {
            assert!(!shows_completion_date(state));
        }
        assert!(shows_rewards(DelegationState::Active, "100"));
        assert!(!shows_rewards(DelegationState::Active, "0"));
        assert!(!shows_rewards(DelegationState::Inactive, "100"));
    }

    #[test]
    fn test_delegation_actions_follow_state_and_chain_config() {
        use GemDelegationAction::*;
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active),
            vec![Stake, Unstake, Redelegate]
        );
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Cosmos, StakeProviderType::Stake, DelegationState::Inactive),
            vec![Unstake, Redelegate]
        );
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Solana, StakeProviderType::Stake, DelegationState::Active),
            vec![Unstake]
        );
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Solana, StakeProviderType::Stake, DelegationState::AwaitingWithdrawal),
            vec![Withdraw]
        );
        assert!(delegation_actions(WalletType::Multicoin, Chain::Cosmos, StakeProviderType::Stake, DelegationState::Pending).is_empty());
        assert!(delegation_actions(WalletType::View, Chain::Cosmos, StakeProviderType::Stake, DelegationState::Active).is_empty());
        assert!(delegation_actions(WalletType::Multicoin, Chain::Bitcoin, StakeProviderType::Stake, DelegationState::Active).is_empty());
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Ethereum, StakeProviderType::Earn, DelegationState::Active),
            vec![Deposit, Withdraw]
        );
        assert_eq!(
            delegation_actions(WalletType::Multicoin, Chain::Ethereum, StakeProviderType::Earn, DelegationState::Inactive),
            vec![Withdraw]
        );
    }

    #[test]
    fn test_stake_actions_follow_the_wallet_chain_and_balance() {
        use GemStakeAction::*;
        let actions = |chain, has_validators, frozen, rewards| {
            stake_actions(WalletType::Multicoin, chain, has_validators, frozen, rewards)
                .into_iter()
                .map(|item| (item.action, item.is_enabled, item.requires_frozen_balance))
                .collect::<Vec<_>>()
        };

        assert_eq!(actions(Chain::Cosmos, true, "0", "0"), vec![(Stake, true, false)]);
        assert_eq!(actions(Chain::Cosmos, false, "0", "0"), vec![(Stake, false, false)]);
        assert_eq!(actions(Chain::Cosmos, true, "0", "5"), vec![(Stake, true, false), (ClaimRewards, true, false)]);
        assert_eq!(
            actions(Chain::Tron, true, "0", "0"),
            vec![(Stake, true, true), (Freeze, true, false), (Unfreeze, true, false)],
            "a freeze chain with nothing frozen asks for a frozen balance before staking"
        );
        assert_eq!(
            actions(Chain::Tron, false, "10", "0"),
            vec![(Stake, false, false), (Freeze, true, false), (Unfreeze, true, false)]
        );
        assert!(stake_actions(WalletType::View, Chain::Cosmos, true, "0", "5").is_empty());
        assert!(
            stake_actions(WalletType::Multicoin, Chain::Bitcoin, true, "0", "5").is_empty(),
            "a chain without staking has no actions"
        );
    }

    #[test]
    fn test_can_claim_all_rewards() {
        assert!(can_claim_all_rewards(Chain::Cosmos, 3));
        assert!(!can_claim_all_rewards(Chain::Sui, 3));
        assert!(can_claim_all_rewards(Chain::Sui, 1));
        assert!(!can_claim_all_rewards(Chain::Bitcoin, 2));
    }

    #[test]
    fn test_can_claim_rewards() {
        assert!(can_claim_rewards(WalletType::Multicoin, Chain::Cosmos, DelegationState::Active, "10"));
        assert!(!can_claim_rewards(WalletType::Multicoin, Chain::Cosmos, DelegationState::Active, "0"));
        assert!(!can_claim_rewards(WalletType::Multicoin, Chain::Cosmos, DelegationState::Inactive, "10"));
        assert!(!can_claim_rewards(WalletType::View, Chain::Cosmos, DelegationState::Active, "10"));
        assert!(!can_claim_rewards(WalletType::Multicoin, Chain::Solana, DelegationState::Active, "10"));
    }

    #[test]
    fn test_recommended_validator_prefers_configured_ids() {
        let recommended = recommended_validator_ids(Chain::Cosmos);
        assert!(!recommended.is_empty());
        let validators = vec![validator("other"), validator(&recommended[0])];
        assert_eq!(recommended_validator(Chain::Cosmos, validators).unwrap().id, recommended[0]);
        assert_eq!(recommended_validator(Chain::Cosmos, vec![validator("other")]).unwrap().id, "other");
        assert!(recommended_validator(Chain::Cosmos, vec![]).is_none());
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
        assert!(requires_frozen_balance(Chain::Tron, "0"));
        assert!(!requires_frozen_balance(Chain::Tron, "10"));
        assert!(!requires_frozen_balance(Chain::Cosmos, "0"));
        assert!(!requires_frozen_balance(Chain::Bitcoin, "0"));
    }

    #[test]
    fn test_claiming_needs_rewards_on_a_chain_that_claims() {
        assert!(can_claim_stake_rewards(Chain::Cosmos, "10"));
        assert!(!can_claim_stake_rewards(Chain::Cosmos, "0"));
        assert!(!can_claim_stake_rewards(Chain::Bitcoin, "10"));
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
        assert_eq!(stake_balance(0, 0, 100, 20, 5).staked_value(Chain::Cosmos), GemBigInt::from(125));
        assert_eq!(stake_balance(0, 0, 100, 20, 0).staked_value(Chain::Cosmos), GemBigInt::from(120));
        assert_eq!(stake_balance(0, 0, 700, 30, 3).staked_value(Chain::Solana), GemBigInt::from(733));
        assert_eq!(stake_balance(9, 9, 100, 0, 0).staked_value(Chain::Cosmos), GemBigInt::from(100));
    }

    #[test]
    fn test_unclaimed_rewards_alone_are_a_staked_position() {
        let rewards_only = stake_balance(0, 0, 0, 0, 7);
        assert_eq!(rewards_only.staked_value(Chain::Cosmos), GemBigInt::from(7));
        assert!(rewards_only.shows_stake_balance(Chain::Cosmos, false));
    }

    #[test]
    fn test_staked_value_uses_the_frozen_balance_on_freeze_chains() {
        assert_eq!(stake_balance(40, 60, 0, 10, 5).staked_value(Chain::Tron), GemBigInt::from(115));
        assert_eq!(stake_balance(40, 60, 999, 0, 0).staked_value(Chain::Tron), GemBigInt::from(100));
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
    fn test_stake_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemStakeBalance) -> (GemBigInt, GemBigInt, GemBigInt, GemBigInt, GemBigInt) =
            |balance| (balance.frozen, balance.locked, balance.staked, balance.pending, balance.rewards);
        assert_eq!(stake_balance(0, 0, 100, 20, 5).staked_value(Chain::Cosmos), GemBigInt::from(125));
    }
}
