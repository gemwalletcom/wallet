pub mod error;
pub mod model;
mod rules;
pub mod sign_message;
pub mod signer;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use gem_wallet_connect::validate_sign_message_account;
use primitives::{Account, ApplicationMetadata, Chain, Wallet, WalletConnection, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

use crate::services::assets::GemAssetsService;
use crate::services::error::GemServiceError;
use crate::services::simulation::GemSimulationService;
use crate::services::wallet_session::GemWalletSessionService;
use crate::wallet_connect::{WalletConnect, WalletConnectAction, WalletConnectChainOperation, WalletConnectTransactionType};

pub use error::GemWalletConnectError;
pub use model::{
    GemSessionApproval, GemSessionProposal, GemWalletConnectAuthAccount, GemWalletConnectFailure, GemWalletConnectMessageRequest, GemWalletConnectOutcome, GemWalletConnectResponse, GemWalletConnectRpcError,
    GemWalletConnectSessionRequest, GemWalletConnectTransactionAction, GemWalletConnectTransactionRequest,
};
pub use sign_message::{GemSignMessagePreview, GemSignMessageService};
pub use signer::GemWalletConnectSigner;
pub use store::GemConnectionStore;

#[derive(uniffi::Object)]
pub struct GemWalletConnectService {
    wallet_connect: WalletConnect,
    simulation: Arc<GemSimulationService>,
    store: Arc<dyn GemConnectionStore>,
    signer: Arc<dyn GemWalletConnectSigner>,
    session: Arc<GemWalletSessionService>,
    assets: Arc<GemAssetsService>,
    seen_messages: Mutex<Vec<String>>,
}

const SEEN_MESSAGES_LIMIT: usize = 512;

#[uniffi::export]
impl GemWalletConnectService {
    #[uniffi::constructor]
    pub fn new(
        simulation: Arc<GemSimulationService>,
        store: Arc<dyn GemConnectionStore>,
        signer: Arc<dyn GemWalletConnectSigner>,
        session: Arc<GemWalletSessionService>,
        assets: Arc<GemAssetsService>,
    ) -> Self {
        Self {
            wallet_connect: WalletConnect::new(),
            simulation,
            store,
            signer,
            session,
            assets,
            seen_messages: Mutex::new(Vec::new()),
        }
    }

    pub fn should_process_message(&self, message_id: String) -> bool {
        let mut seen = self.seen_messages.lock().expect("wallet connect seen messages lock");
        rules::record_seen_message(&mut seen, message_id, SEEN_MESSAGES_LIMIT)
    }

    pub fn is_origin_rejected(&self, metadata_url: String, origin: Option<String>, validation: WalletConnectionVerificationStatus) -> bool {
        rules::is_origin_rejected(&self.wallet_connect.validate_origin(metadata_url, origin, validation))
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

    pub fn config_session_properties(&self, properties: HashMap<String, String>, caip2_chains: Vec<String>, accounts: Vec<Account>) -> HashMap<String, String> {
        self.wallet_connect.config_session_properties(properties, caip2_chains, accounts)
    }

    pub fn authentication_chain_ids(&self, chain_ids: Vec<String>) -> Vec<String> {
        rules::authentication_chain_ids(&chain_ids)
    }

    pub fn authentication_accounts(&self, chain_ids: Vec<String>, wallet: Wallet) -> Vec<GemWalletConnectAuthAccount> {
        rules::authentication_accounts(&chain_ids, &wallet)
    }

    pub fn authentication_methods(&self) -> Vec<String> {
        rules::authentication_methods()
    }

    pub async fn has_sessions(&self) -> Result<bool, GemServiceError> {
        Ok(!self.store.get_sessions().await?.is_empty())
    }

    pub async fn delete_session(&self, session_id: String) -> Result<(), GemServiceError> {
        self.store.delete_sessions(vec![session_id]).await
    }

    pub async fn prepare_session_proposal(
        &self,
        required_chain_ids: Vec<String>,
        optional_chain_ids: Vec<String>,
        metadata: ApplicationMetadata,
        origin: Option<String>,
        validation: WalletConnectionVerificationStatus,
    ) -> Result<GemSessionProposal, GemWalletConnectError> {
        let wallets = self.session.get_wallets().await?;
        let current_wallet_id = self.session.get_current_wallet_id()?;
        let required = rules::parse_chains(&required_chain_ids).ok_or(GemWalletConnectError::UnsupportedChains)?;
        let optional = rules::parse_known_chains(&optional_chain_ids);
        let verification_status = self.wallet_connect.validate_origin(metadata.url.clone(), origin, validation);
        if rules::is_origin_rejected(&verification_status) {
            return Err(GemWalletConnectError::InvalidOrigin);
        }
        let wallets = rules::session_wallets(wallets, &required, &optional);
        let default_wallet = rules::default_wallet(&wallets, current_wallet_id).ok_or(GemWalletConnectError::UnsupportedWallets)?;
        Ok(GemSessionProposal {
            proposal: WalletConnectionSessionProposal {
                default_wallet,
                wallets,
                metadata,
            },
            verification_status,
        })
    }

    pub fn application_metadata(&self, name: String, description: String, url: String, icons: Vec<String>) -> ApplicationMetadata {
        rules::application_metadata(name, description, url, icons)
    }

    pub fn session_approval(&self, wallet: Wallet) -> GemSessionApproval {
        let chains = rules::session_chains(&wallet, &rules::supported_chains());
        let accounts = wallet.accounts.into_iter().filter(|account| chains.contains(&account.chain)).collect();
        GemSessionApproval {
            chains,
            accounts,
            methods: rules::session_methods(),
            events: rules::session_events(),
        }
    }

    pub fn session(&self, topic: String, accounts: Vec<String>, expire_at: i64, metadata: ApplicationMetadata) -> Result<WalletConnectionSession, GemServiceError> {
        let chains = rules::account_chains(&accounts);
        let expire_at = DateTime::<Utc>::from_timestamp(expire_at, 0).ok_or_else(|| GemServiceError::InvalidInput {
            msg: format!("invalid session expiry {expire_at}"),
        })?;
        Ok(rules::session(topic, chains, expire_at, metadata))
    }

    pub fn user_rejected_error(&self) -> GemWalletConnectRpcError {
        rules::user_rejected_error()
    }

    pub async fn process_request(&self, request: GemWalletConnectSessionRequest) -> GemWalletConnectOutcome {
        if !self.should_process_message(rules::request_message_id(&request.topic, &request.request_id)) {
            return GemWalletConnectOutcome::ignored();
        }
        let expiry = request.expiry;
        if rules::is_expired(expiry, Utc::now()) {
            return GemWalletConnectOutcome::expired();
        }
        let Ok(connection) = self.connection(&request.topic).await else {
            return GemWalletConnectOutcome::rejected(None);
        };
        let domain = connection.session.metadata.url;
        if self.is_origin_rejected(domain.clone(), request.origin, request.validation) {
            return GemWalletConnectOutcome::rejected(Some(GemWalletConnectFailure::MaliciousOrigin));
        }
        let Some(chain_id) = request.chain_id else {
            return GemWalletConnectOutcome::rejected(None);
        };
        match self.handle_request(request.topic, request.method, request.params, chain_id, domain, expiry).await {
            Ok(response) => GemWalletConnectOutcome {
                response: Some(response),
                failure: None,
            },
            Err(_) if rules::is_expired(expiry, Utc::now()) => GemWalletConnectOutcome::expired(),
            Err(GemServiceError::Cancelled) => GemWalletConnectOutcome::rejected(None),
            Err(error) => GemWalletConnectOutcome::rejected(Some(GemWalletConnectFailure::Failed { message: error.to_string() })),
        }
    }
}

impl GemWalletConnectService {
    pub fn validate_origin(&self, metadata_url: String, origin: Option<String>, validation: WalletConnectionVerificationStatus) -> WalletConnectionVerificationStatus {
        self.wallet_connect.validate_origin(metadata_url, origin, validation)
    }
}

impl GemWalletConnectService {
    async fn handle_request(
        &self,
        topic: String,
        method: String,
        params: String,
        chain_id: String,
        domain: String,
        expiry: Option<u64>,
    ) -> Result<GemWalletConnectResponse, GemServiceError> {
        let action = self.wallet_connect.parse_request(topic.clone(), method, params, chain_id, domain.clone())?;
        let session_id = topic;
        let response = match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                let (connection, account) = self.connection_account(&session_id, chain).await?;
                validate_sign_message_account(&sign_type.clone().into(), &data, &account.address).map_err(|msg| GemServiceError::InvalidInput { msg })?;
                let simulation = self.simulation.simulate_sign_message(chain, sign_type.clone(), data.clone(), domain).await?;
                let assets = self.assets.ensure_simulation_assets(simulation.asset_ids()).await?;
                let message = self.wallet_connect.decode_sign_message(chain, sign_type, data);
                self.ensure_live(expiry)?;
                let signature = self
                    .signer
                    .sign_message(GemWalletConnectMessageRequest {
                        session_id,
                        chain,
                        wallet: connection.wallet,
                        account,
                        session: connection.session,
                        simulation,
                        message,
                        assets,
                    })
                    .await?;
                self.wallet_connect.encode_sign_message(chain, signature)
            }
            WalletConnectAction::SignTransaction { chain, transaction_type, data } => {
                let transaction_id = self
                    .sign_transaction(session_id, chain, transaction_type, data, GemWalletConnectTransactionAction::Sign, expiry)
                    .await?;
                self.wallet_connect.encode_sign_transaction(chain, transaction_id)
            }
            WalletConnectAction::SignAllTransactions {
                chain,
                transaction_type,
                transactions,
            } => {
                let [data] = transactions.as_slice() else {
                    return Err(GemServiceError::Unsupported {
                        msg: "signAllTransactions with multiple transactions is not yet supported".to_string(),
                    });
                };
                let signed = self
                    .sign_transaction(session_id, chain, transaction_type, data.clone(), GemWalletConnectTransactionAction::Sign, expiry)
                    .await?;
                self.wallet_connect.encode_sign_all_transactions(vec![signed])
            }
            WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
                let transaction_id = self
                    .sign_transaction(session_id, chain, transaction_type, data, GemWalletConnectTransactionAction::Send, expiry)
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
                    WalletConnectChainOperation::GetChainId => GemWalletConnectResponse::Error {
                        error: rules::method_not_found_error(),
                    },
                });
            }
            WalletConnectAction::Unsupported { .. } => {
                return Ok(GemWalletConnectResponse::Error {
                    error: rules::method_not_found_error(),
                });
            }
        };
        Ok(GemWalletConnectResponse::Response { value: response })
    }

    async fn sign_transaction(
        &self,
        session_id: String,
        chain: Chain,
        transaction_type: WalletConnectTransactionType,
        data: String,
        action: GemWalletConnectTransactionAction,
        expiry: Option<u64>,
    ) -> Result<String, GemServiceError> {
        let (connection, account) = self.connection_account(&session_id, chain).await?;
        let transaction = self.wallet_connect.decode_send_transaction(transaction_type.clone(), data.clone())?;
        rules::validate_transaction_sender(&transaction, &account)?;
        let simulation = self.simulation.simulate_send_transaction(chain, transaction_type, data).await?;
        let transfer = rules::transfer_data(chain, connection.session.metadata.clone(), transaction, action)?;
        self.ensure_live(expiry)?;
        self.signer
            .sign_transaction(GemWalletConnectTransactionRequest {
                session_id,
                chain,
                wallet: connection.wallet,
                account,
                session: connection.session,
                simulation,
                transfer,
                action,
            })
            .await
    }

    fn ensure_live(&self, expiry: Option<u64>) -> Result<(), GemServiceError> {
        match rules::is_expired(expiry, Utc::now()) {
            true => Err(GemServiceError::Cancelled),
            false => Ok(()),
        }
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
        self.store.get_connection(session_id.to_string()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("WalletConnect session {session_id} not found"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::{TestWalletConnectSigner, make_service};
    use super::*;
    use crate::wallet_connect::WalletConnectResponseType;
    use futures::executor::block_on;
    use num_bigint::BigUint;
    use primitives::ApprovalData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;

    fn request(request_id: &str) -> GemWalletConnectSessionRequest {
        GemWalletConnectSessionRequest {
            topic: "topic".to_string(),
            request_id: request_id.to_string(),
            method: "personal_sign".to_string(),
            params: r#"["0x48656c6c6f", "0xaddress"]"#.to_string(),
            chain_id: Some("eip155:1".to_string()),
            origin: Some("https://example.com".to_string()),
            validation: WalletConnectionVerificationStatus::Verified,
            expiry: None,
        }
    }

    fn rejected() -> Option<GemWalletConnectResponse> {
        Some(GemWalletConnectResponse::Error {
            error: rules::user_rejected_error(),
        })
    }

    #[test]
    fn test_process_tron_approval_request() {
        block_on(async {
            for (params, owner, spender, contract, value, is_unlimited) in [
                (
                    include_str!("../../../../crates/gem_wallet_connect/testdata/tron_send_transaction.json"),
                    "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM",
                    "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM",
                    "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
                    BigUint::from(0u32),
                    false,
                ),
                (
                    include_str!("../../../../crates/gem_tron/testdata/wallet_connect_permit2_approval.json"),
                    "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7",
                    "TQqgNg13s2DjvXhW1ky4v6TsR8wZGvb7Y4",
                    "TTJxU3P8rHycAyFY4kVtGNfmnMH4ezcuM9",
                    BigUint::from_bytes_be(&[0xff; 20]),
                    true,
                ),
            ] {
                for (method, action) in [
                    ("tron_signTransaction", GemWalletConnectTransactionAction::Sign),
                    ("tron_sendTransaction", GemWalletConnectTransactionAction::Send),
                ] {
                    let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Tron, owner)]);
                    let mut service = make_service(Ok("{}".to_string()), wallet).await;
                    let signer = Arc::new(TestWalletConnectSigner {
                        result: Ok("{}".to_string()),
                        transactions: Mutex::new(Vec::new()),
                    });
                    service.signer = signer.clone();
                    let outcome = service
                        .process_request(GemWalletConnectSessionRequest {
                            method: method.to_string(),
                            params: params.to_string(),
                            chain_id: Some("tron:0x2b6653dc".to_string()),
                            ..request(method)
                        })
                        .await;
                    assert_eq!(outcome.failure, None);
                    let requests = signer.transactions.lock().unwrap();
                    assert_eq!(requests.len(), 1);
                    let request = &requests[0];
                    assert_eq!(request.action, action);
                    assert_eq!(request.transfer.recipient.address, contract);
                    assert_eq!(
                        request.transfer.input_type.get_generic_data().unwrap().approval,
                        Some(ApprovalData {
                            token: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
                            spender: spender.to_string(),
                            value: value.clone(),
                            is_unlimited,
                        })
                    );
                    assert_eq!(request.simulation.header.as_ref().unwrap().asset_id.to_string(), "tron_TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
                    assert_eq!(request.simulation.payload[0].value, spender);
                    assert_eq!(request.simulation.payload[1].value, contract);
                    assert_eq!(request.simulation.header.as_ref().unwrap().is_unlimited, is_unlimited);
                    assert_eq!(request.simulation.warnings.len(), 1);
                }
            }
        });
    }

    #[test]
    fn test_process_siws_request() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Solana, TEST_PRIVATE_KEY_SOLANA_ADDRESS)]);
        let service = block_on(make_service(Ok("signature".to_string()), wallet));
        assert_eq!(
            block_on(service.process_request(GemWalletConnectSessionRequest::mock_siws())),
            GemWalletConnectOutcome {
                response: Some(GemWalletConnectResponse::Response {
                    value: WalletConnectResponseType::Object {
                        json: r#"{"signature":"signature"}"#.to_string()
                    },
                }),
                failure: None,
            }
        );
    }

    #[test]
    fn test_process_siws_request_account_mismatch() {
        let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Solana, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]);
        let service = block_on(make_service(
            Err(GemServiceError::Platform {
                msg: "signer callback reached".to_string(),
            }),
            wallet,
        ));
        assert_eq!(
            block_on(service.process_request(GemWalletConnectSessionRequest::mock_siws())),
            GemWalletConnectOutcome {
                response: rejected(),
                failure: Some(GemWalletConnectFailure::Failed {
                    message: "SIWS address does not match signing account".to_string()
                }),
            }
        );
    }

    #[test]
    fn test_process_request_outcomes() {
        block_on(async {
            let service = make_service(Ok("0xsignature".to_string()), Wallet::mock()).await;

            let signed = service.process_request(request("1")).await;
            assert_eq!(
                signed,
                GemWalletConnectOutcome {
                    response: Some(GemWalletConnectResponse::Response {
                        value: WalletConnectResponseType::String { value: "0xsignature".to_string() },
                    }),
                    failure: None,
                }
            );

            let duplicate = service.process_request(request("1")).await;
            assert_eq!(
                duplicate,
                GemWalletConnectOutcome { response: None, failure: None },
                "a redelivered request gets no second answer: the first one is still being decided or was already sent"
            );

            let expired = service
                .process_request(GemWalletConnectSessionRequest {
                    expiry: Some((Utc::now() - chrono::TimeDelta::seconds(1)).timestamp() as u64),
                    ..request("expired")
                })
                .await;
            assert_eq!(
                expired,
                GemWalletConnectOutcome {
                    response: Some(GemWalletConnectResponse::Error {
                        error: rules::request_expired_error(),
                    }),
                    failure: Some(GemWalletConnectFailure::Expired),
                },
                "a request the dapp stopped waiting for is answered as expired and never reaches the signer"
            );

            let live = service
                .process_request(GemWalletConnectSessionRequest {
                    expiry: Some((Utc::now() + chrono::TimeDelta::seconds(300)).timestamp() as u64),
                    ..request("live")
                })
                .await;
            assert_eq!(live.failure, None, "a request still inside its expiry is answered normally");

            let malicious = service
                .process_request(GemWalletConnectSessionRequest {
                    validation: WalletConnectionVerificationStatus::Malicious,
                    ..request("2")
                })
                .await;
            assert_eq!(malicious.failure, Some(GemWalletConnectFailure::MaliciousOrigin));
            assert_eq!(malicious.response, rejected());

            let unknown_session = service
                .process_request(GemWalletConnectSessionRequest {
                    topic: "other".to_string(),
                    ..request("4")
                })
                .await;
            assert_eq!(
                unknown_session,
                GemWalletConnectOutcome {
                    response: rejected(),
                    failure: None
                }
            );

            let no_chain = service.process_request(GemWalletConnectSessionRequest { chain_id: None, ..request("5") }).await;
            assert_eq!(
                no_chain,
                GemWalletConnectOutcome {
                    response: rejected(),
                    failure: None
                }
            );

            let unsupported = service
                .process_request(GemWalletConnectSessionRequest {
                    method: "eth_chainId".to_string(),
                    ..request("6")
                })
                .await;
            assert_eq!(
                unsupported.response,
                Some(GemWalletConnectResponse::Error {
                    error: rules::method_not_found_error()
                })
            );
            assert_eq!(unsupported.failure, None);

            let cancelled = make_service(Err(GemServiceError::Cancelled), Wallet::mock()).await.process_request(request("7")).await;
            assert_eq!(
                cancelled,
                GemWalletConnectOutcome {
                    response: rejected(),
                    failure: None
                },
                "the user's own cancel is not an error"
            );

            let failed = make_service(Err(GemServiceError::Platform { msg: "keystore".to_string() }), Wallet::mock())
                .await
                .process_request(request("8"))
                .await;
            assert_eq!(failed.response, rejected());
            assert_eq!(failed.failure, Some(GemWalletConnectFailure::Failed { message: "keystore".to_string() }));
        });
    }
}
