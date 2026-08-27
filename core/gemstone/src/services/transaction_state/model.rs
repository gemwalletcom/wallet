use primitives::{AssetId, Chain, Transaction, TransactionId, TransactionState, Wallet};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionStateUpdate {
    pub state: TransactionState,
    pub fee: Option<String>,
    pub block_number: Option<String>,
    pub metadata: Option<String>,
    pub confirmation_eta_seconds: Option<u32>,
}

impl GemTransactionStateUpdate {
    pub fn new(state: TransactionState) -> Self {
        Self {
            state,
            fee: None,
            block_number: None,
            metadata: None,
            confirmation_eta_seconds: None,
        }
    }

    pub fn has_field_changes(&self) -> bool {
        self.fee.is_some() || self.block_number.is_some() || self.metadata.is_some() || self.confirmation_eta_seconds.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionStateResult {
    pub transaction_id: TransactionId,
    pub state: TransactionState,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPendingTransaction {
    pub wallet: Wallet,
    pub transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GemTransactionPostProcessing {
    pub balance_asset_ids: Vec<AssetId>,
    pub stake_chains: Vec<Chain>,
    pub earn_asset_ids: Vec<AssetId>,
    pub sync_nfts: bool,
}
