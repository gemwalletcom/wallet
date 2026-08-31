use serde::{Deserialize, Serialize};

use crate::models::custom_types::GemBigInt;
use crate::models::gateway::GemFeeRate;
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::balance::GemAssetBalance;
use crate::services::price::GemAssetPrice;
use crate::services::transfer::GemTransferData;
use primitives::FeePriority;
use primitives::{Account, AssetId, SimulationResult, Transaction, Wallet};

pub type GemAccount = Account;

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub from: GemAccount,
    pub transfer: GemTransferData,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSendResult {
    pub hashes: Vec<String>,
    pub transactions: Vec<Transaction>,
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
