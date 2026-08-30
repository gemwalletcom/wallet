use primitives::{Chain, DelegationState, DelegationValidator, StakeProviderType, WalletType};

use super::model::GemDelegationAction;
use super::rules;

#[derive(Default, uniffi::Object)]
pub struct GemStakeConfigService {}

#[uniffi::export]
impl GemStakeConfigService {
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

    pub fn can_claim_stake_rewards(&self, chain: Chain, rewards_amount: String) -> bool {
        rules::can_claim_stake_rewards(chain, &rewards_amount)
    }

    pub fn requires_frozen_balance(&self, chain: Chain, frozen_amount: String) -> bool {
        rules::requires_frozen_balance(chain, &frozen_amount)
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
}
