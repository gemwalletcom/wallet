pub mod model;
mod rules;
pub mod signer;
pub mod store;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use primitives::{
    Account, ApplicationMetadata, Chain, SimulationResult, Wallet, WalletConnection, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus,
    WalletId,
};

use crate::services::error::GemServiceError;
use crate::transaction_simulation::TransactionSimulationService;
use crate::wallet_connect::{WalletConnect, WalletConnectAction, WalletConnectChainOperation, WalletConnectTransactionType};

pub use model::{
    GemSessionApproval, GemSessionProposal, GemSessionWallets, GemWalletConnectRequest, GemWalletConnectResponse, GemWalletConnectSignPayload, GemWalletConnectSignRequest,
    GemWalletConnectTransactionAction,
};
pub use signer::GemWalletConnectSigner;
pub use store::GemConnectionStore;

#[derive(uniffi::Object)]
pub struct GemWalletConnectService {
    wallet_connect: WalletConnect,
    simulation: Arc<TransactionSimulationService>,
    store: Arc<dyn GemConnectionStore>,
    signer: Arc<dyn GemWalletConnectSigner>,
}

#[uniffi::export]
impl GemWalletConnectService {
    #[uniffi::constructor]
    pub fn new(simulation: Arc<TransactionSimulationService>, store: Arc<dyn GemConnectionStore>, signer: Arc<dyn GemWalletConnectSigner>) -> Self {
        Self {
            wallet_connect: WalletConnect::new(),
            simulation,
            store,
            signer,
        }
    }

    pub async fn add_connection(&self, connection: WalletConnection) -> Result<(), GemServiceError> {
        self.store.add_connection(connection).await
    }

    pub async fn update_sessions(&self, sessions: Vec<WalletConnectionSession>) -> Result<(), GemServiceError> {
        let local = self.store.get_sessions().await?;
        let delete_ids = rules::sessions_to_delete(&local, &sessions);
        if !delete_ids.is_empty() {
            self.store.delete_sessions(delete_ids).await?;
        }
        for session in rules::sessions_to_update(&local, sessions) {
            self.store.update_session(session).await?;
        }
        Ok(())
    }

    pub async fn has_sessions(&self) -> Result<bool, GemServiceError> {
        Ok(!self.store.get_sessions().await?.is_empty())
    }

    pub async fn delete_session(&self, session_id: String) -> Result<(), GemServiceError> {
        self.store.delete_sessions(vec![session_id]).await
    }

    pub fn prepare_session_proposal(
        &self,
        wallets: Vec<Wallet>,
        current_wallet_id: Option<WalletId>,
        required_chain_ids: Vec<String>,
        optional_chain_ids: Vec<String>,
        metadata: ApplicationMetadata,
        origin: Option<String>,
        validation: WalletConnectionVerificationStatus,
    ) -> Result<GemSessionProposal, GemServiceError> {
        let required = rules::parse_chains(&self.wallet_connect, &required_chain_ids).ok_or_else(|| GemServiceError::Status {
            msg: "unsupported chains".to_string(),
        })?;
        let optional = rules::parse_known_chains(&self.wallet_connect, &optional_chain_ids);
        let verification_status = self.wallet_connect.validate_origin(metadata.url.clone(), origin, validation);
        if matches!(
            verification_status,
            WalletConnectionVerificationStatus::Invalid | WalletConnectionVerificationStatus::Malicious
        ) {
            return Err(GemServiceError::Status {
                msg: "invalid origin".to_string(),
            });
        }
        let session_wallets = self
            .select_session_wallets(wallets, current_wallet_id, required, optional)
            .ok_or_else(|| GemServiceError::Status {
                msg: "wallets unsupported".to_string(),
            })?;
        Ok(GemSessionProposal {
            proposal: WalletConnectionSessionProposal {
                default_wallet: session_wallets.default_wallet,
                wallets: session_wallets.wallets,
                metadata,
            },
            verification_status,
        })
    }

    pub fn application_metadata(&self, name: String, description: String, url: String, icons: Vec<String>) -> ApplicationMetadata {
        rules::application_metadata(name, description, url, icons)
    }

    pub fn session_approval(&self, wallet: Wallet, supported_chains: Vec<Chain>) -> GemSessionApproval {
        let chains = rules::session_chains(&wallet, &supported_chains);
        let accounts = wallet.accounts.into_iter().filter(|account| chains.contains(&account.chain)).collect();
        GemSessionApproval {
            chains,
            accounts,
            methods: rules::session_methods(),
            events: rules::session_events(),
        }
    }

    pub fn session(&self, topic: String, accounts: Vec<String>, expire_at: i64, metadata: ApplicationMetadata) -> Result<WalletConnectionSession, GemServiceError> {
        let chains = rules::account_chains(&self.wallet_connect, &accounts);
        let expire_at = DateTime::<Utc>::from_timestamp(expire_at, 0).ok_or_else(|| GemServiceError::Status {
            msg: format!("invalid session expiry {expire_at}"),
        })?;
        Ok(rules::session(topic, chains, expire_at, metadata))
    }

    pub async fn handle_request(&self, request: GemWalletConnectRequest) -> Result<GemWalletConnectResponse, GemServiceError> {
        let action = self
            .wallet_connect
            .parse_request(request.topic.clone(), request.method, request.params, request.chain_id, request.domain.clone())?;
        let session_id = request.topic;
        let response = match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                let (connection, account) = self.connection_account(&session_id, chain).await?;
                let simulation = self.simulation.simulate_sign_message(chain, sign_type.clone(), data.clone(), request.domain).await?;
                let message = self.wallet_connect.decode_sign_message(chain, sign_type, data);
                let payload = GemWalletConnectSignPayload::Message { message };
                let signature = self.signer.sign(sign_request(session_id, chain, connection, account, simulation, payload)).await?;
                self.wallet_connect.encode_sign_message(chain, signature)
            }
            WalletConnectAction::SignTransaction { chain, transaction_type, data } => {
                let transaction_id = self
                    .sign_transaction(session_id, chain, transaction_type, data, GemWalletConnectTransactionAction::Sign)
                    .await?;
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
                let signed = self
                    .sign_transaction(session_id, chain, transaction_type, data.clone(), GemWalletConnectTransactionAction::Sign)
                    .await?;
                self.wallet_connect.encode_sign_all_transactions(vec![signed])
            }
            WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
                let transaction_id = self
                    .sign_transaction(session_id, chain, transaction_type, data, GemWalletConnectTransactionAction::Send)
                    .await?;
                self.wallet_connect.encode_send_transaction(chain, transaction_id)
            }
            WalletConnectAction::GetAccounts { chain } => {
                let accounts = self.get_accounts(&session_id, chain).await?;
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

impl GemWalletConnectService {
    fn select_session_wallets(
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

    async fn sign_transaction(
        &self,
        session_id: String,
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
        action: GemWalletConnectTransactionAction,
    ) -> Result<String, GemServiceError> {
        let (connection, account) = self.connection_account(&session_id, chain).await?;
        let simulation = self.simulation.simulate_send_transaction(chain, transaction_type.clone(), data.clone()).await?;
        let transaction = self.wallet_connect.decode_send_transaction(transaction_type, data)?;
        let payload = GemWalletConnectSignPayload::Transaction { transaction, action };
        self.signer.sign(sign_request(session_id, chain, connection, account, simulation, payload)).await
    }

    async fn get_accounts(&self, session_id: &str, chain: Chain) -> Result<Vec<Account>, GemServiceError> {
        let connection = self.connection(session_id).await?;
        rules::validate_session_chain(&connection.session, chain)?;
        Ok(connection.wallet.accounts.into_iter().filter(|account| account.chain == chain).collect())
    }

    async fn connection_account(&self, session_id: &str, chain: Chain) -> Result<(WalletConnection, Account), GemServiceError> {
        let connection = self.connection(session_id).await?;
        let account = rules::session_account(&connection, chain)?;
        Ok((connection, account))
    }

    async fn connection(&self, session_id: &str) -> Result<WalletConnection, GemServiceError> {
        self.store.get_connection(session_id.to_string()).await?.ok_or_else(|| GemServiceError::Status {
            msg: format!("WalletConnect session {session_id} not found"),
        })
    }
}

fn sign_request(
    session_id: String,
    chain: Chain,
    connection: WalletConnection,
    account: Account,
    simulation: SimulationResult,
    payload: GemWalletConnectSignPayload,
) -> GemWalletConnectSignRequest {
    GemWalletConnectSignRequest {
        session_id,
        chain,
        wallet: connection.wallet,
        account,
        session: connection.session,
        simulation,
        payload,
    }
}
