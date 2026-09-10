use super::rules;
use crate::models::custom_types::GemBigInt;
use crate::services::amount::model::GemAmountType;
use crate::services::amount::rules as amount_rules;
use crate::services::error::GemServiceError;
use crate::services::transfer::GemTransferData;
use primitives::{Delegation, DelegationValidator, Resource, StakeType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDelegationAction {
    Stake,
    Unstake,
    Redelegate,
    Withdraw,
    Deposit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemStakeAction {
    Stake,
    Freeze,
    Unfreeze,
    ClaimRewards,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemStakeActionItem {
    pub action: GemStakeAction,
    pub is_enabled: bool,
    pub requires_frozen_balance: bool,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemDelegationDestination {
    Details,
    Withdraw { transfer: GemTransferData },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemClaimRewards {
    pub value: GemBigInt,
    pub destination: GemClaimRewardsDestination,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemClaimRewardsDestination {
    Transfer { transfer: GemTransferData },
    Amount { delegations: Vec<Delegation> },
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemStakeAmountInput {
    Stake {
        validators: Vec<DelegationValidator>,
        validator: Option<DelegationValidator>,
    },
    Redelegate {
        validators: Vec<DelegationValidator>,
        delegation: Delegation,
        validator: Option<DelegationValidator>,
    },
    Unstake {
        delegation: Delegation,
    },
    Withdraw {
        delegation: Delegation,
    },
    Rewards {
        delegations: Vec<Delegation>,
        validator: Option<DelegationValidator>,
    },
    Freeze {
        resource: Resource,
    },
    Unfreeze {
        resource: Resource,
    },
}

#[uniffi::export]
impl GemStakeAmountInput {
    pub fn amount_type(&self) -> GemAmountType {
        amount_rules::stake_amount_type(self)
    }

    pub fn stake_type(&self) -> Result<StakeType, GemServiceError> {
        rules::stake_type(self)
    }

    pub fn with_validator(&self, validator: DelegationValidator) -> GemStakeAmountInput {
        rules::with_validator(self, validator)
    }

    pub fn with_resource(&self, resource: Resource) -> GemStakeAmountInput {
        rules::with_resource(self, resource)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemStakeValidatorSelection {
    pub options: Vec<DelegationValidator>,
    pub recommended: Vec<DelegationValidator>,
    pub validator: Option<DelegationValidator>,
    pub can_select: bool,
}
