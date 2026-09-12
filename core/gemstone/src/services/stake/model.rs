use super::rules;
use crate::models::custom_types::GemBigInt;
use crate::services::amount::model::GemAmountType;
use crate::services::amount::rules as amount_rules;
use crate::services::error::GemServiceError;
use crate::services::transfer::GemTransferData;
use primitives::{Delegation, DelegationState, DelegationValidator, Resource, StakeType, YieldProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDelegationTone {
    Positive,
    Pending,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDelegationCompletion {
    ActiveIn,
    AvailableIn,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemDelegationStatus {
    pub state: DelegationState,
    pub tone: GemDelegationTone,
    pub completion: Option<GemDelegationCompletion>,
}

#[uniffi::export]
pub fn delegation_status(delegation: Delegation) -> GemDelegationStatus {
    rules::delegation_status(&delegation)
}

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
    pub options: Vec<GemValidatorRow>,
    pub recommended: Vec<GemValidatorRow>,
    pub validator: Option<GemValidatorRow>,
    pub can_select: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemValidatorRow {
    pub validator: DelegationValidator,
    pub name: String,
    pub image_url: String,
    pub placeholder: String,
    pub provider: Option<YieldProvider>,
}
