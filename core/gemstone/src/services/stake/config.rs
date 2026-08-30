use primitives::{Chain, DelegationState, DelegationValidator, StakeProviderType, WalletType};

use super::model::{GemDelegationAction, GemStakeBalance};
use super::rules;
use crate::models::custom_types::GemBigInt;

#[derive(Default, uniffi::Object)]
pub struct StakeConfig {}

#[uniffi::export]
impl StakeConfig {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn delegation_actions(&self, wallet_type: WalletType, chain: Chain, provider: StakeProviderType, state: DelegationState) -> Vec<GemDelegationAction> {
        rules::delegation_actions(wallet_type, chain, provider, state)
    }

    pub fn can_claim_delegation_rewards(&self, wallet_type: WalletType, chain: Chain, state: DelegationState, rewards: String) -> bool {
        rules::can_claim_rewards(wallet_type, chain, state, &rewards)
    }

    pub fn validator_explorer_address(&self, validator: DelegationValidator) -> Option<String> {
        rules::validator_explorer_address(&validator)
    }

    pub fn shows_completion_date(&self, state: DelegationState) -> bool {
        rules::shows_completion_date(state)
    }

    pub fn shows_rewards(&self, state: DelegationState, rewards: String) -> bool {
        rules::shows_rewards(state, &rewards)
    }

    pub fn can_claim_stake_rewards(&self, chain: Chain, rewards_value: String) -> bool {
        rules::can_claim_stake_rewards(chain, &rewards_value)
    }

    pub fn requires_frozen_balance(&self, chain: Chain, frozen_value: String) -> bool {
        rules::requires_frozen_balance(chain, &frozen_value)
    }

    pub fn recommended_validator_ids(&self, chain: Chain) -> Vec<String> {
        rules::recommended_validator_ids(chain)
    }

    pub fn recommended_validator(&self, chain: Chain, validators: Vec<DelegationValidator>) -> Option<DelegationValidator> {
        rules::recommended_validator(chain, validators)
    }

    pub fn selectable_validators(&self, validators: Vec<DelegationValidator>) -> Vec<DelegationValidator> {
        rules::selectable_validators(validators)
    }

    pub fn staked_value(&self, chain: Chain, balance: GemStakeBalance) -> GemBigInt {
        rules::staked_value(chain, &balance)
    }

    pub fn shows_stake_balance(&self, chain: Chain, is_stake_enabled: bool, balance: GemStakeBalance) -> bool {
        rules::shows_stake_balance(chain, is_stake_enabled, &balance)
    }
}
