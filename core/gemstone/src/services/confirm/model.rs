use super::error::GemConfirmError;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::gateway::GemFeeRate;
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::balance::GemAssetBalance;
use crate::services::transfer::GemTransferData;
use crate::transfer_amount::GemTransferAmount;
use primitives::AssetPrice;
use primitives::{
    Account, AddressName, Asset, AssetId, Chain, ChainAddress, FeePriority, FeeUnitType, SimulationPayloadField, SimulationPayloadFieldType, SimulationResult, SimulationWarning,
    Transaction, Wallet,
};

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
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmData {
    pub input: GemConfirmInput,
    pub fee: GemTransactionLoadFee,
    pub selected_priority: FeePriority,
    pub fee_rates: Vec<GemFeeRate>,
    pub metadata: GemTransactionLoadMetadata,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemExecuteResult {
    Signed { data: Vec<String> },
    Sent { hashes: Vec<String>, transactions: Vec<Transaction> },
}

pub(super) struct GemSendResult {
    pub(super) hashes: Vec<String>,
    pub(super) transactions: Vec<Transaction>,
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
    pub unit_value: GemBigInt,
    pub fee: Option<GemBigInt>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeRateRows {
    pub rows: Vec<GemFeeRateRow>,
    pub unit_type: FeeUnitType,
    pub unit_decimals: u32,
    pub supports_custom_fee: bool,
    pub selected_total: Option<GemBigInt>,
    pub normal_total: Option<GemBigInt>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeAsset {
    pub asset: Asset,
    pub balance: GemAssetBalance,
    pub price: Option<AssetPrice>,
}

impl GemConfirmSimulation {
    pub(super) fn address_requests(&self, chain: Chain) -> Vec<ChainAddress> {
        self.primary_fields
            .iter()
            .chain(self.secondary_fields.iter())
            .filter(|field| field.field_type == SimulationPayloadFieldType::Address)
            .map(|field| ChainAddress::new(chain, field.value.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoad {
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
    pub warnings: Vec<SimulationWarning>,
    pub simulation: Option<GemConfirmSimulation>,
    pub address_names: Vec<AddressName>,
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

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemApprovalValue {
    Exact { value: GemBigUint },
    Unlimited,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSimulationValue {
    pub asset: Asset,
    pub value: GemApprovalValue,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSimulationBalanceChange {
    pub asset: Asset,
    pub value: GemBigInt,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmSimulation {
    pub primary_fields: Vec<SimulationPayloadField>,
    pub secondary_fields: Vec<SimulationPayloadField>,
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConfirmScreen {
    pub phase: GemConfirmPhase,
    pub amount_failed: bool,
    pub has_critical_warning: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemConfirmButtonKind {
    Confirm,
    Retry,
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

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemConfirmFeeRow {
    Loading,
    Ready,
    Unavailable,
}

#[uniffi::export]
impl GemConfirmScreen {
    pub fn button(&self) -> GemConfirmButton {
        super::rules::confirm_button(self)
    }

    pub fn fee_row(&self) -> GemConfirmFeeRow {
        super::rules::confirm_fee_row(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_pairs_each_balance_with_its_own_price() {
        let balance = |chain: primitives::Chain| GemAssetBalance {
            asset_id: AssetId::from_chain(chain),
            ..GemAssetBalance::mock()
        };
        let price = |chain: primitives::Chain, value: f64| AssetPrice {
            asset_id: AssetId::from_chain(chain),
            price: value,
            price_change_percentage_24h: 0.0,
            updated_at: chrono::Utc::now(),
        };
        let metadata = GemConfirmMetadata {
            asset_balance: balance(primitives::Chain::Solana),
            fee_asset_balance: balance(primitives::Chain::Bitcoin),
            prices: vec![price(primitives::Chain::Bitcoin, 2.0), price(primitives::Chain::Solana, 1.0)],
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
