use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;
use primitives::{Wallet, WalletConnection, WalletConnectionSession, WalletConnectionVerificationStatus};

use super::{GemConnectionStore, GemWalletConnectMessageRequest, GemWalletConnectService, GemWalletConnectSessionRequest, GemWalletConnectSigner, GemWalletConnectTransactionRequest};
use crate::alien::AlienProvider;
use crate::api::GemDeviceApiClient;
use crate::keystore::GemKeystore;
use crate::services::GemScanService;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::node::GemNodeService;
use crate::services::simulation::GemSimulationService;
use crate::services::wallet::testkit::{MemoryAddressStore, MemoryKeystorePassword, MemoryWalletStore};
use crate::services::wallet_connect::sign_message::GemSignMessageService;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

#[derive(Default)]
pub struct MemoryConnectionStore {
    pub connections: Mutex<Vec<WalletConnection>>,
}

#[async_trait]
impl GemConnectionStore for MemoryConnectionStore {
    async fn get_connection(&self, session_id: String) -> Result<Option<WalletConnection>, GemServiceError> {
        Ok(self.connections.lock().unwrap().iter().find(|connection| connection.session.id == session_id).cloned())
    }
    async fn get_sessions(&self) -> Result<Vec<WalletConnectionSession>, GemServiceError> {
        Ok(self.connections.lock().unwrap().iter().map(|connection| connection.session.clone()).collect())
    }
    async fn add_connection(&self, connection: WalletConnection) -> Result<(), GemServiceError> {
        self.connections.lock().unwrap().push(connection);
        Ok(())
    }
    async fn update_session(&self, session: WalletConnectionSession) -> Result<(), GemServiceError> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.iter_mut().find(|connection| connection.session.id == session.id) {
            connection.session = session;
        }
        Ok(())
    }
    async fn delete_sessions(&self, session_ids: Vec<String>) -> Result<(), GemServiceError> {
        self.connections.lock().unwrap().retain(|connection| !session_ids.contains(&connection.session.id));
        Ok(())
    }
}

pub struct TestWalletConnectSigner {
    pub result: Result<String, GemServiceError>,
    pub transactions: Mutex<Vec<GemWalletConnectTransactionRequest>>,
    pub messages: Mutex<Vec<GemWalletConnectMessageRequest>>,
}

impl TestWalletConnectSigner {
    pub fn new(result: Result<String, GemServiceError>) -> Self {
        Self {
            result,
            transactions: Mutex::new(Vec::new()),
            messages: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl GemWalletConnectSigner for TestWalletConnectSigner {
    async fn sign_message(&self, request: GemWalletConnectMessageRequest) -> Result<String, GemServiceError> {
        self.messages.lock().unwrap().push(request);
        self.result.clone()
    }
    async fn sign_transaction(&self, request: GemWalletConnectTransactionRequest) -> Result<String, GemServiceError> {
        self.transactions.lock().unwrap().push(request);
        self.result.clone()
    }
}

impl GemWalletConnectSessionRequest {
    pub fn mock(request_id: &str) -> Self {
        Self {
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

    pub fn mock_siws() -> Self {
        let message = include_str!("../../../../crates/gem_solana/testdata/siws_sign_in.txt");
        let data = bs58::encode(message).into_string();
        Self {
            method: "solana_signMessage".to_string(),
            params: format!(r#"{{"message":"{data}","pubkey":"{TEST_PRIVATE_KEY_SOLANA_ADDRESS}"}}"#),
            chain_id: Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string()),
            ..Self::mock("siws")
        }
    }
}

impl GemWalletConnectService {
    pub async fn mock(signer: Result<String, GemServiceError>, wallet: Wallet) -> Self {
        Self::mock_with_signer(Arc::new(TestWalletConnectSigner::new(signer)), wallet, Arc::new(TestAlienProvider::with_status(200))).await
    }

    pub async fn mock_with_signer(signer: Arc<TestWalletConnectSigner>, wallet: Wallet, provider: Arc<TestAlienProvider>) -> Self {
        let store = Arc::new(MemoryConnectionStore::default());
        let chains: Vec<_> = wallet.accounts.iter().map(|account| account.chain).collect();
        store
            .add_connection(WalletConnection {
                session: WalletConnectionSession::mock("topic", &chains),
                wallet,
            })
            .await
            .unwrap();
        let provider: Arc<dyn AlienProvider> = provider;
        let wallet_session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), Arc::new(MemoryWalletStore::default())));
        let sign_message = Arc::new(GemSignMessageService::new(
            Arc::new(GemNameService::new(
                Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
                Arc::new(MemoryAddressStore::default()),
            )),
            Arc::new(GemExplorerService::mock()),
            GemKeystore::new(std::env::temp_dir().to_string_lossy().to_string()).unwrap(),
            Arc::new(MemoryKeystorePassword::default()),
        ));
        Self::new(
            Arc::new(GemSimulationService::new(provider.clone(), Arc::new(GemNodeService::mock()))),
            Arc::new(GemScanService::new(Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))))),
            store,
            signer,
            wallet_session,
            Arc::new(GemAssetsService::mock(provider, Arc::new(MemoryAssetStore::default()))),
            sign_message,
            primitives::Platform::IOS,
        )
    }
}
