use serde::{Deserialize, Serialize};

use crate::models::gateway::GemFeeRate;
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::transfer::GemTransferData;
use primitives::{Account, AssetId, ScanTransaction, SimulationResult, Transaction, Wallet};

pub type GemAccount = Account;

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub from: GemAccount,
    pub transfer: GemTransferData,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemConfirmFeeSelection {
    Priority { priority: String },
    Custom { gas_price: String },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmLoadOptions {
    pub fee_selection: GemConfirmFeeSelection,
    pub fee_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConfirmData {
    pub fee: GemTransactionLoadFee,
    pub selected_priority: String,
    pub fee_rates: Vec<GemFeeRate>,
    pub metadata: GemTransactionLoadMetadata,
    pub scan: Option<ScanTransaction>,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemExecuteResult {
    Signed { data: Vec<String> },
    Sent { hashes: Vec<String>, transactions: Vec<Transaction> },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSendResult {
    pub hashes: Vec<String>,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSendInput {
    pub wallet: Wallet,
    pub transfer: GemTransferData,
    pub value: String,
    pub fee: GemTransactionLoadFee,
    pub network_fee: String,
    pub metadata: GemTransactionLoadMetadata,
    pub simulation: Option<SimulationResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAcquireAssetFlow {
    Options,
    Fiat,
}
