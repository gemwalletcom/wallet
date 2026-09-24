use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::GemListRow;
use crate::services::amount::model::GemAmountType;
use crate::services::amount::rules as amount_rules;
use crate::services::balance::GemAssetBalance;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::transfer::GemTransferData;
use primitives::{Asset, BalanceMetadata, BlockExplorerLink, Currency, Delegation, DelegationState, DelegationValidator, EarnType, Resource, StakeType, WalletType, YieldProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemStakeSection {
    Manage,
    Resources,
    Delegations,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemDelegationStatus {
    pub state: DelegationState,
    pub tone: GemValueTone,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationDetails {
    pub title: GemLocalizedText,
    pub header: GemDelegationListRow,
    pub actions: Vec<GemDelegationAction>,
    pub balance: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub rewards: Option<GemFormattedNumber>,
    pub rewards_fiat: Option<GemFormattedNumber>,
    pub rows: Vec<GemListRow>,
    pub claim: Option<GemTransferData>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemDelegationListRow {
    pub validator: GemValidatorRow,
    pub status: GemDelegationStatus,
    pub balance: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub rewards: Option<GemFormattedNumber>,
    pub rewards_fiat: Option<GemFormattedNumber>,
}

#[uniffi::export]
pub fn delegation_details(wallet_type: WalletType, delegation: Delegation, asset: Asset, price: Option<f64>, currency: Currency) -> GemDelegationDetails {
    rules::delegation_details(wallet_type, &delegation, &asset, price, currency, Vec::new())
}

#[uniffi::export]
pub fn delegation_list_row(delegation: Delegation, asset: Asset, price: Option<f64>, currency: Currency) -> GemDelegationListRow {
    rules::delegation_list_row(&delegation, &asset, price, currency)
}

#[uniffi::export]
pub fn delegation_list_rows(delegations: Vec<Delegation>, asset: Asset, price: Option<f64>, currency: Currency) -> Vec<GemDelegationListRow> {
    delegations.iter().map(|delegation| rules::delegation_list_row(delegation, &asset, price, currency.clone())).collect()
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeActionItem {
    pub action: GemStakeAction,
    pub row: GemListRow,
    pub tap: GemStakeActionTap,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemStakeActionTap {
    Open { destination: GemStakeDestination },
    FrozenBalanceInfo,
    Disabled,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemStakeDestination {
    Amount { input: GemStakeAmountInput },
    Confirm { transfer: GemTransferData },
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemDelegationDestination {
    Details,
    Confirm { transfer: GemTransferData },
    Amount { asset: Asset, input: GemDelegationAmountInput },
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemDelegationAmountInput {
    Stake { input: GemStakeAmountInput },
    Earn { earn_type: EarnType },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeInput {
    pub wallet_type: WalletType,
    pub asset: Asset,
    pub balance: GemAssetBalance,
    pub balance_metadata: Option<BalanceMetadata>,
    pub staking_apr: Option<f64>,
    pub price: Option<f64>,
    pub currency: Currency,
    pub validators: Vec<DelegationValidator>,
    pub delegations: Vec<Delegation>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeDelegationItem {
    pub delegation: Delegation,
    pub row: GemDelegationListRow,
    pub destination: GemDelegationDestination,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeViewState {
    pub sections: Vec<GemStakeSection>,
    pub info_rows: Vec<GemListRow>,
    pub actions: Vec<GemStakeActionItem>,
    pub resource_rows: Vec<GemListRow>,
    pub delegations: Vec<GemStakeDelegationItem>,
    pub validators: Vec<DelegationValidator>,
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

    pub fn resource(&self) -> Option<Resource> {
        match self {
            Self::Freeze { resource } | Self::Unfreeze { resource } => Some(*resource),
            Self::Stake { .. } | Self::Redelegate { .. } | Self::Unstake { .. } | Self::Withdraw { .. } | Self::Rewards { .. } => None,
        }
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
pub struct GemEarnActions {
    pub deposit_provider: Option<DelegationValidator>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemValidatorRow {
    pub validator: DelegationValidator,
    pub name: String,
    pub image_url: String,
    pub placeholder: String,
    pub provider: Option<YieldProvider>,
    pub apr: GemLocalizedText,
    pub explorer: Option<BlockExplorerLink>,
}
