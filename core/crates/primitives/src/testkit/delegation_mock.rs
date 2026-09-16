use crate::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};
use num_bigint::BigUint;

impl Delegation {
    pub fn mock() -> Self {
        Delegation {
            base: DelegationBase::mock(),
            validator: DelegationValidator::mock(),
            price: None,
        }
    }

    pub fn mock_base(base: DelegationBase) -> Self {
        Delegation {
            base,
            validator: DelegationValidator::mock(),
            price: None,
        }
    }

    pub fn mock_with(chain: Chain, provider_type: StakeProviderType, state: DelegationState, rewards: u64) -> Self {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(chain),
                state,
                rewards: BigUint::from(rewards),
                ..DelegationBase::mock()
            },
            validator: DelegationValidator {
                chain,
                provider_type,
                ..DelegationValidator::mock()
            },
            price: None,
        }
    }

    pub fn mock_with_validator(validator: DelegationValidator) -> Self {
        Delegation {
            base: DelegationBase::mock_with_validator(&validator.id),
            validator,
            price: None,
        }
    }

    pub fn mock_tron(validator_id: &str) -> Self {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Tron),
                state: DelegationState::Active,
                balance: BigUint::from(0u32),
                shares: BigUint::from(0u32),
                rewards: BigUint::from(0u32),
                completion_date: None,
                delegation_id: validator_id.to_string(),
                validator_id: validator_id.to_string(),
            },
            validator: DelegationValidator::mock_tron(validator_id),
            price: None,
        }
    }

    pub fn mock_osmosis(validator_id: &str) -> Self {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Osmosis),
                state: DelegationState::Active,
                balance: BigUint::from(10u32),
                shares: BigUint::from(0u32),
                rewards: BigUint::from(0u32),
                completion_date: None,
                delegation_id: "25053096".to_string(),
                validator_id: validator_id.to_string(),
            },
            validator: DelegationValidator::mock_osmosis(validator_id),
            price: None,
        }
    }

    pub fn mock_with_id(delegation_id: String) -> Self {
        Delegation::mock_base(DelegationBase::mock_with_id(delegation_id))
    }
}

impl DelegationBase {
    pub fn mock() -> Self {
        DelegationBase {
            asset_id: AssetId::from_chain(Chain::Sui),
            state: DelegationState::Active,
            balance: BigUint::from(1000000000u64),
            shares: BigUint::from(1000000000u64),
            rewards: BigUint::from(0u64),
            completion_date: None,
            delegation_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            validator_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }
    }

    pub fn mock_with_balance(balance: u64, rewards: u64) -> Self {
        DelegationBase {
            balance: BigUint::from(balance),
            rewards: BigUint::from(rewards),
            ..Self::mock()
        }
    }

    pub fn mock_with_validator(validator_id: &str) -> Self {
        DelegationBase {
            delegation_id: validator_id.to_string(),
            validator_id: validator_id.to_string(),
            ..Self::mock()
        }
    }

    pub fn mock_with_id(delegation_id: String) -> Self {
        DelegationBase { delegation_id, ..Self::mock() }
    }
}

impl DelegationValidator {
    pub fn mock() -> Self {
        DelegationValidator::stake(Chain::Sui, "validator1".to_string(), "Test Validator".to_string(), true, 0.05, 0.08)
    }

    pub fn mock_cosmos(id: &str) -> Self {
        DelegationValidator::stake(Chain::Cosmos, id.to_string(), id.to_string(), true, 0.0, 1.0)
    }

    pub fn mock_tron(id: &str) -> Self {
        DelegationValidator::stake(Chain::Tron, id.to_string(), id.to_string(), true, 0.0, 0.0)
    }

    pub fn mock_osmosis(id: &str) -> Self {
        DelegationValidator::stake(Chain::Osmosis, id.to_string(), String::new(), true, 1.0, 9.0)
    }
}
