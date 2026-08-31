use async_trait::async_trait;
use primitives::Wallet;

use crate::GemstoneError;
use crate::models::transaction::{GemSignedTransaction, GemSignerInput};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemTransactionSigner: Send + Sync {
    async fn sign(&self, wallet: Wallet, input: GemSignerInput) -> Result<Vec<GemSignedTransaction>, GemstoneError>;
}
