use std::collections::{HashMap, HashSet};

use crate::services::collections::{stale, unique};

use std::str::FromStr;

use num_bigint::BigUint;
use primitives::AddressName;
use primitives::{AddressType, Chain, DelegationBase, DelegationState, DelegationValidator, StakeChain, StakeProviderType, VerificationStatus, WalletType};
use rand::seq::IndexedRandom;

use super::model::GemDelegationAction;
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

pub fn inactive_validator(chain: Chain, id: String, name: String) -> DelegationValidator {
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
}
