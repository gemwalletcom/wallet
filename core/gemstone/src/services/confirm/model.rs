use serde::{Deserialize, Serialize};

use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::gateway::GemFeeRate;
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::balance::GemAssetBalance;
use crate::services::price::GemAssetPrice;
use crate::services::transfer::GemTransferData;
use crate::transfer_amount::{GemTransferAmount, GemTransferAmountError};
use primitives::{
    Account, AddressName, Asset, AssetId, Chain, ChainAddress, FeePriority, SimulationPayloadField, SimulationPayloadFieldType, SimulationResult, Transaction, Wallet,
};

pub type GemAccount = Account;

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub from: GemAccount,
    pub transfer: GemTransferData,
}

#[derive(uniffi::Record)]
pub struct GemConfirmInitialState {
    pub fee_priority: FeePriority,
    pub fee_asset: Asset,
    pub metadata: Option<GemConfirmMetadata>,
    pub simulation: Option<GemConfirmSimulation>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemConfirmFeeSelection {
    Priority { priority: FeePriority },
    Custom { gas_price: GemBigInt },
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSendInput {
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
    pub prices: Vec<GemAssetPrice>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeAsset {
    pub asset: Asset,
    pub balance: GemAssetBalance,
    pub price: Option<GemAssetPrice>,
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
    pub fee_assets: Vec<GemFeeAsset>,
    pub preload: GemConfirmPreload,
    pub simulation: GemConfirmSimulationState,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmSimulationState {
    pub simulation: Option<GemConfirmSimulation>,
    pub address_names: Vec<AddressName>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransferAmountResult {
    Amount { amount: GemTransferAmount },
    Error { error: GemTransferAmountError, asset: Asset },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmPreload {
    pub confirm_data: GemConfirmData,
    pub metadata: GemConfirmMetadata,
    pub fee_asset: Asset,
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
