use async_trait::async_trait;
use primitives::{Account, Chain, SimulationResult};

use crate::message::sign_type::SignMessage;
use crate::services::error::GemServiceError;
use crate::wallet_connect::WalletConnectTransaction;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletConnectSigner: Send + Sync {
    async fn sign_message(&self, session_id: String, chain: Chain, message: SignMessage, simulation: SimulationResult) -> Result<String, GemServiceError>;
    async fn sign_transaction(&self, session_id: String, chain: Chain, transaction: WalletConnectTransaction, simulation: SimulationResult) -> Result<String, GemServiceError>;
    async fn send_transaction(&self, session_id: String, chain: Chain, transaction: WalletConnectTransaction, simulation: SimulationResult) -> Result<String, GemServiceError>;
    async fn get_accounts(&self, session_id: String, chain: Chain) -> Result<Vec<Account>, GemServiceError>;
}
