use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::GemListRow;
use crate::services::amount::model::GemAmountType;
use crate::services::amount::rules as amount_rules;
use crate::services::assets::icon::GemAssetIcon;
use crate::services::assets::model::{GemAssetText, GemValueHeader};
use crate::services::localization::GemLocalizedText;
use crate::services::transfer::GemTransferData;
use primitives::{Asset, AssetData, BlockExplorerLink, Currency, Delegation, DelegationState, DelegationValidator, EarnType, Resource, StakeType, WalletType, YieldProvider};

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
    pub icon: GemAssetIcon,
    pub value_header: GemValueHeader,
    pub header: GemDelegationListRow,
    pub actions: Vec<GemDelegationActionItem>,
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
    rules::delegation_details(wallet_type, &delegation, &asset, price, currency, Vec::new(), &[])
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDelegationActionItem {
    pub action: GemDelegationAction,
    pub destination: GemDelegationDestination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemStakeActionKind {
    Stake,
    Freeze,
    Unfreeze,
    ClaimRewards,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeActionItem {
    pub kind: GemStakeActionKind,
    pub row: GemListRow,
    pub action: GemStakeAction,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemStakeAction {
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
    pub asset_data: AssetData,
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
    pub asset: GemAssetText,
    pub sections: Vec<GemStakeSection>,
    pub info_rows: Vec<GemListRow>,
    pub actions: Vec<GemStakeActionItem>,
    pub resource_rows: Vec<GemListRow>,
    pub delegations: Vec<GemStakeDelegationItem>,
    pub docs_url: Option<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemStakeAmountInput {
    Stake { validator: DelegationValidator },
    Redelegate { delegation: Delegation, validator: DelegationValidator },
    Unstake { delegation: Delegation },
    Withdraw { delegation: Delegation },
    Rewards { delegations: Vec<Delegation>, validator: DelegationValidator },
    Freeze { resource: Resource },
    Unfreeze { resource: Resource },
}

#[uniffi::export]
impl GemStakeAmountInput {
    pub fn amount_type(&self) -> GemAmountType {
        amount_rules::stake_amount_type(self)
    }

    pub fn with_validator(&self, validator: DelegationValidator) -> GemStakeAmountInput {
        rules::with_validator(self, validator)
    }

    pub fn with_resource(&self, resource: Resource) -> GemStakeAmountInput {
        rules::with_resource(self, resource)
    }
}

impl GemStakeAmountInput {
    pub fn stake_type(&self) -> StakeType {
        rules::stake_type(self)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemStakeAmountSelection {
    Validator { validator: GemValidatorRow, can_select: bool },
    Resource { options: Vec<Resource>, selected: Resource },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemStakeValidatorOptions {
    pub recommended: Vec<GemValidatorRow>,
    pub options: Vec<GemValidatorRow>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemEarnView {
    pub asset: GemAssetText,
    pub apr_row: GemListRow,
    pub providers: Vec<DelegationValidator>,
    pub deposit_provider: Option<DelegationValidator>,
    pub positions: Vec<GemStakeDelegationItem>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemEarnInput {
    pub wallet_type: WalletType,
    pub asset: Asset,
    pub providers: Vec<DelegationValidator>,
    pub delegations: Vec<Delegation>,
    pub asset_apr: Option<f64>,
    pub price: Option<f64>,
    pub currency: Currency,
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
