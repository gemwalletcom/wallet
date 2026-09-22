use super::error::GemConfirmError;
use super::rules::approval_value_from;
use crate::formatted_number::GemValueTone;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::gateway::GemFeeRate;
use crate::models::list::GemListRow;
use crate::models::transaction::{GemFeeOptionItem, GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::balance::GemAssetBalance;
use crate::services::error_text::GemErrorText;
use crate::services::localization::GemLocalizedText;
use crate::services::simulation::{GemSimulationPayloadRow, address_requests, named_payload_rows};
use crate::services::transactions::GemAmountSign;
use crate::services::transfer::GemTransferData;
use crate::services::transfer::model::GemConfirmDestination;
use crate::transfer_amount::GemTransferAmount;
use primitives::AssetPrice;
use primitives::BlockExplorerLink;
use primitives::{Account, AddressName, Asset, AssetId, Chain, ChainAddress, FeePriority, FeeUnitType, SimulationResult, Wallet};

pub type GemAccount = Account;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmInput {
    pub from: GemAccount,
    pub transfer: GemTransferData,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemConfirmFeeSelection {
    Priority { priority: FeePriority },
    Custom { gas_price: GemBigInt },
}

#[uniffi::export]
impl GemConfirmFeeSelection {
    pub fn selected_priority(&self) -> Option<FeePriority> {
        match self {
            Self::Priority { priority } => Some(*priority),
            Self::Custom { .. } => None,
        }
    }

    pub fn custom_gas_price(&self) -> Option<GemBigInt> {
        match self {
            Self::Priority { .. } => None,
            Self::Custom { gas_price } => Some(gas_price.clone()),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoadOptions {
    pub fee_selection: GemConfirmFeeSelection,
    pub fee_asset_id: Option<AssetId>,
    pub asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmData {
    pub input: GemConfirmInput,
    pub fee: GemTransactionLoadFee,
    pub additional_fees: Vec<GemFeeOptionItem>,
    pub selected_priority: FeePriority,
    pub fee_rates: Vec<GemFeeRate>,
    pub metadata: GemTransactionLoadMetadata,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemSubmitResult {
    Signed { data: Vec<String>, warning: Option<GemErrorText> },
    Sent { hashes: Vec<String>, warning: Option<GemErrorText> },
}

#[derive(Debug, Clone)]
pub struct SendInput {
    pub wallet: Wallet,
    pub confirm: GemConfirmData,
    pub value: GemBigInt,
    pub network_fee: GemBigInt,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAcquireAssetFlow {
    Options,
    Fiat,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConfirmMetadata {
    pub asset_balance: GemAssetBalance,
    pub fee_asset_balance: GemAssetBalance,
    pub prices: Vec<AssetPrice>,
}

#[uniffi::export]
impl GemConfirmMetadata {
    pub fn price(&self, asset_id: AssetId) -> Option<AssetPrice> {
        self.prices.iter().find(|price| price.asset_id == asset_id).cloned()
    }

    pub fn asset_price(&self) -> Option<AssetPrice> {
        self.price(self.asset_balance.asset_id.clone())
    }

    pub fn fee_price(&self) -> Option<AssetPrice> {
        self.price(self.fee_asset_balance.asset_id.clone())
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeRateRow {
    pub priority: FeePriority,
    pub fee: Option<GemBigInt>,
    pub value: GemLocalizedText,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeRateRows {
    pub rows: Vec<GemFeeRateRow>,
    pub shows_options: bool,
    pub unit_type: FeeUnitType,
    pub unit_decimals: u32,
    pub supports_custom_fee: bool,
    pub selected_total: Option<GemBigInt>,
    pub normal_total: Option<GemBigInt>,
    pub custom_rate: Option<GemLocalizedText>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeAsset {
    pub asset: Asset,
    pub balance: GemAssetBalance,
    pub price: Option<AssetPrice>,
}

impl GemConfirmSimulation {
    pub(super) fn address_requests(&self, chain: Chain) -> Vec<ChainAddress> {
        [address_requests(&self.primary_fields, chain), address_requests(&self.secondary_fields, chain)].concat()
    }

    pub(super) fn with_address_names(self, names: &[AddressName]) -> Self {
        Self {
            primary_fields: named_payload_rows(self.primary_fields, names),
            secondary_fields: named_payload_rows(self.secondary_fields, names),
            ..self
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoad {
    pub transfer: GemTransferData,
    pub sender: GemAccount,
    pub fee_asset: Asset,
    pub metadata: GemConfirmMetadata,
    pub fee_assets: Vec<GemFeeAsset>,
    pub simulation: GemConfirmSimulationState,
    pub address_name: Option<AddressName>,
    pub preload: Option<GemConfirmPreload>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmSimulationState {
    pub chain: Chain,
    pub result: Option<SimulationResult>,
    pub warnings: Vec<GemListRow>,
    pub simulation: Option<GemConfirmSimulation>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransferAmountResult {
    Amount { amount: GemTransferAmount },
    Error { error: GemConfirmError },
}

#[derive(Debug, Clone)]
pub struct GemConfirmFeeLoad {
    pub fee_asset: Asset,
    pub metadata: GemConfirmMetadata,
    pub preload: GemConfirmPreload,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmPreload {
    pub confirm_data: GemConfirmData,
    pub amount: GemTransferAmountResult,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemApprovalValue {
    Exact { value: GemBigUint },
    Unlimited,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSimulationValue {
    pub asset: Asset,
    pub value: GemApprovalValue,
}

impl GemSimulationValue {
    pub(crate) fn from_simulation(simulation: &SimulationResult, assets: &[Asset]) -> Option<Self> {
        let header = simulation.valid_header()?;
        let asset = assets.iter().find(|asset| asset.id == header.asset_id)?.clone();
        Some(Self {
            asset,
            value: approval_value_from(header.value.as_ref(), header.is_unlimited),
        })
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSimulationBalanceChange {
    pub asset: Asset,
    pub value: GemBigInt,
    pub sign: GemAmountSign,
    pub tone: GemValueTone,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmSimulation {
    pub primary_fields: Vec<GemSimulationPayloadRow>,
    pub secondary_fields: Vec<GemSimulationPayloadRow>,
    pub header: Option<GemSimulationValue>,
    pub balance_changes: Vec<GemSimulationBalanceChange>,
    pub has_critical_warning: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemConfirmPhase {
    Loading,
    Ready,
    Confirming,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemConfirmStage {
    Load,
    Execute,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmFailure {
    pub stage: GemConfirmStage,
    pub error: GemConfirmError,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmScreen {
    pub phase: GemConfirmPhase,
    pub has_critical_warning: bool,
    pub failure: Option<GemConfirmFailure>,
    #[uniffi(default = true)]
    pub has_preload: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemConfirmAction {
    Load,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemConfirmButtonKind {
    Confirm,
    Retry,
    AccountMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemConfirmButtonState {
    Disabled,
    Loading,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConfirmButton {
    pub kind: GemConfirmButtonKind,
    pub state: GemConfirmButtonState,
}

#[uniffi::export]
pub fn shows_fee_assets(fee_asset_ids: Vec<AssetId>, selected: Option<AssetId>) -> bool {
    super::rules::shows_fee_assets(&fee_asset_ids, selected.as_ref())
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmFeeRow {
    Loading,
    Ready,
    Unavailable { text: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_pairs_each_balance_with_its_own_price() {
        let now = chrono::Utc::now();
        let metadata = GemConfirmMetadata {
            asset_balance: GemAssetBalance::zero(AssetId::from_chain(primitives::Chain::Solana)),
            fee_asset_balance: GemAssetBalance::zero(AssetId::from_chain(primitives::Chain::Bitcoin)),
            prices: vec![
                AssetPrice::new(AssetId::from_chain(primitives::Chain::Bitcoin), 2.0, 0.0, now),
                AssetPrice::new(AssetId::from_chain(primitives::Chain::Solana), 1.0, 0.0, now),
            ],
        };

        assert_eq!(metadata.asset_price().map(|price| price.price), Some(1.0));
        assert_eq!(metadata.fee_price().map(|price| price.price), Some(2.0));
        assert_eq!(metadata.price(AssetId::from_chain(primitives::Chain::Ethereum)), None);
    }

    #[test]
    fn test_fee_selection_answers_only_for_its_own_case() {
        let priority = GemConfirmFeeSelection::Priority { priority: FeePriority::Fast };
        let custom = GemConfirmFeeSelection::Custom { gas_price: 7.into() };

        assert_eq!(priority.selected_priority(), Some(FeePriority::Fast));
        assert_eq!(priority.custom_gas_price(), None);
        assert_eq!(custom.selected_priority(), None);
        assert_eq!(custom.custom_gas_price(), Some(7.into()));
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmRowContent {
    Row {
        row: GemListRow,
    },
    Recipient {
        destination: GemConfirmDestination,
        name: Option<String>,
        address: String,
        memo: Option<String>,
        chain: Chain,
        link: BlockExplorerLink,
        avatar: Option<GemAvatar>,
        is_selectable: bool,
    },
    Details,
    PaymentAsset {
        symbol: String,
        selectable: bool,
    },
}

/// What a contact shows next to a recipient: its picture when it has one, its initials otherwise.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAvatar {
    pub image_url: Option<String>,
    pub initials: String,
}
