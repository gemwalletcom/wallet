pub mod model;
mod rules;
pub mod signer;

use std::sync::Arc;

use primitives::{Chain, Wallet, WalletId};

use crate::services::error::GemServiceError;
use crate::transaction_simulation::TransactionSimulationService;
use crate::wallet_connect::{WalletConnect, WalletConnectAction, WalletConnectChainOperation};

pub use model::{GemSessionWallets, GemWalletConnectRequest, GemWalletConnectResponse};
pub use signer::GemWalletConnectSigner;

#[derive(uniffi::Object)]
pub struct GemWalletConnectService {
    wallet_connect: WalletConnect,
    simulation: Arc<TransactionSimulationService>,
    signer: Arc<dyn GemWalletConnectSigner>,
}

#[uniffi::export]
impl GemWalletConnectService {
    #[uniffi::constructor]
    pub fn new(simulation: Arc<TransactionSimulationService>, signer: Arc<dyn GemWalletConnectSigner>) -> Self {
        Self {
            wallet_connect: WalletConnect::new(),
            simulation,
            signer,
        }
    }

    pub fn select_session_wallets(
        &self,
        wallets: Vec<Wallet>,
        current_wallet_id: Option<WalletId>,
        required_chains: Vec<Chain>,
        optional_chains: Vec<Chain>,
    ) -> Option<GemSessionWallets> {
        let wallets = rules::session_wallets(wallets, &required_chains, &optional_chains);
        let default_wallet = rules::default_wallet(&wallets, current_wallet_id)?;
        Some(GemSessionWallets { default_wallet, wallets })
    }

    pub fn session_chains(&self, wallet: Wallet, supported_chains: Vec<Chain>) -> Vec<Chain> {
        rules::session_chains(&wallet, &supported_chains)
    }

    pub async fn handle_request(&self, request: GemWalletConnectRequest) -> Result<GemWalletConnectResponse, GemServiceError> {
        let action = self
            .wallet_connect
            .parse_request(request.topic.clone(), request.method, request.params, request.chain_id, request.domain.clone())?;
        let session_id = request.topic;
        let response = match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                let simulation = self.simulation.simulate_sign_message(chain, sign_type.clone(), data.clone(), request.domain).await?;
                let message = self.wallet_connect.decode_sign_message(chain, sign_type, data);
                let signature = self.signer.sign_message(session_id, chain, message, simulation).await?;
                self.wallet_connect.encode_sign_message(chain, signature)
            }
            WalletConnectAction::SignTransaction { chain, transaction_type, data } => {
                let simulation = self.simulation.simulate_send_transaction(chain, transaction_type.clone(), data.clone()).await?;
                let transaction = self.wallet_connect.decode_send_transaction(transaction_type, data)?;
                let transaction_id = self.signer.sign_transaction(session_id, chain, transaction, simulation).await?;
                self.wallet_connect.encode_sign_transaction(chain, transaction_id)
            }
            WalletConnectAction::SignAllTransactions {
                chain,
                transaction_type,
                transactions,
            } => {
                let [data] = transactions.as_slice() else {
                    return Err(GemServiceError::Status {
                        msg: "signAllTransactions with multiple transactions is not yet supported".to_string(),
                    });
                };
                let simulation = self.simulation.simulate_send_transaction(chain, transaction_type.clone(), data.clone()).await?;
                let transaction = self.wallet_connect.decode_send_transaction(transaction_type, data.clone())?;
                let signed = self.signer.sign_transaction(session_id, chain, transaction, simulation).await?;
                self.wallet_connect.encode_sign_all_transactions(vec![signed])
            }
            WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
                let simulation = self.simulation.simulate_send_transaction(chain, transaction_type.clone(), data.clone()).await?;
                let transaction = self.wallet_connect.decode_send_transaction(transaction_type, data)?;
                let transaction_id = self.signer.send_transaction(session_id, chain, transaction, simulation).await?;
                self.wallet_connect.encode_send_transaction(chain, transaction_id)
            }
            WalletConnectAction::GetAccounts { chain } => {
                let accounts = self.signer.get_accounts(session_id, chain).await?;
                self.wallet_connect.encode_get_accounts(chain, accounts)
            }
            WalletConnectAction::ChainOperation { operation } => {
                return Ok(match operation {
                    WalletConnectChainOperation::AddChain | WalletConnectChainOperation::SwitchChain { .. } => GemWalletConnectResponse::Null,
                    WalletConnectChainOperation::GetChainId => GemWalletConnectResponse::MethodNotFound,
                });
            }
            WalletConnectAction::Unsupported { .. } => return Ok(GemWalletConnectResponse::MethodNotFound),
        };
        Ok(GemWalletConnectResponse::Response { value: response })
    }
}
