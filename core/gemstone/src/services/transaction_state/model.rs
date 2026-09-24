use crate::models::custom_types::GemBigInt;
use crate::services::failures::StepFailure;
use primitives::{AssetId, Chain, Transaction, TransactionId, TransactionState, WalletId};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionStateUpdate {
    pub state: TransactionState,
    pub fee: Option<GemBigInt>,
    pub block_number: Option<String>,
    pub metadata: Option<String>,
    pub confirmation_eta_seconds: Option<u32>,
    pub asset_ids: Option<Vec<AssetId>>,
}

impl GemTransactionStateUpdate {
    pub fn new(state: TransactionState) -> Self {
        Self {
            state,
            fee: None,
            block_number: None,
            metadata: None,
            confirmation_eta_seconds: None,
            asset_ids: None,
        }
    }

    pub fn has_field_changes(&self) -> bool {
        self.fee.is_some() || self.block_number.is_some() || self.metadata.is_some() || self.confirmation_eta_seconds.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemTransactionStateResult {
    pub transaction_id: TransactionId,
    pub state: TransactionState,
    pub failures: Vec<GemPostProcessingFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPostProcessingStep {
    Balances,
    Stake,
    Earn,
    Nfts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GemPostProcessingFailure {
    pub step: GemPostProcessingStep,
    pub message: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPendingTransaction {
    pub wallet_id: WalletId,
    pub transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionPostProcessing {
    pub balance_asset_ids: Vec<AssetId>,
    pub stake_chains: Vec<Chain>,
    pub earn_asset_ids: Vec<AssetId>,
    pub sync_nfts: bool,
}

impl StepFailure for GemPostProcessingFailure {
    type Step = GemPostProcessingStep;

    fn new(step: GemPostProcessingStep, message: String) -> Self {
        Self { step, message }
    }
}
