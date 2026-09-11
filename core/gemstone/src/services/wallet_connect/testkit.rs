use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use primitives::testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS;
use primitives::{ApplicationMetadata, Wallet, WalletConnection, WalletConnectionSession, WalletConnectionVerificationStatus};

use super::rules;
use super::{
    GemConnectionStore, GemWalletConnectMessageRequest, GemWalletConnectService, GemWalletConnectSessionRequest, GemWalletConnectSigner, GemWalletConnectTransactionRequest,
};
use crate::alien::AlienProvider;
use crate::api::GemApiClient;
use crate::gateway::{EmptyPreferences, GemGateway};
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::simulation::GemSimulationService;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::TestAlienProvider;

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
}

#[async_trait]
impl GemWalletConnectSigner for TestWalletConnectSigner {
    async fn sign_message(&self, _request: GemWalletConnectMessageRequest) -> Result<String, GemServiceError> {
        self.result.clone()
    }
    async fn sign_transaction(&self, request: GemWalletConnectTransactionRequest) -> Result<String, GemServiceError> {
        self.transactions.lock().unwrap().push(request);
        self.result.clone()
    }
}

impl GemWalletConnectSessionRequest {
    pub fn mock_siws() -> Self {
        let message = include_str!("../../../../crates/gem_solana/testdata/siws_sign_in.txt");
        let data = bs58::encode(message).into_string();
        Self {
            topic: "topic".to_string(),
            request_id: "siws".to_string(),
            method: "solana_signMessage".to_string(),
            params: format!(r#"{{"message":"{data}","pubkey":"{TEST_PRIVATE_KEY_SOLANA_ADDRESS}"}}"#),
            chain_id: Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string()),
            origin: Some("https://example.com".to_string()),
            validation: WalletConnectionVerificationStatus::Verified,
        }
    }
}

pub(super) async fn make_service(signer: Result<String, GemServiceError>, wallet: Wallet) -> GemWalletConnectService {
    let store = Arc::new(MemoryConnectionStore::default());
    let chains = wallet.accounts.iter().map(|account| account.chain).collect();
    let session = rules::session("topic".to_string(), chains, Utc::now(), ApplicationMetadata::mock());
    store.add_connection(WalletConnection { session, wallet }).await.unwrap();
    let provider: Arc<dyn AlienProvider> = Arc::new(TestAlienProvider::with_status(200));
    let api = Arc::new(GemApiClient::new(provider.clone()));
    let wallet_session = Arc::new(GemWalletSessionService::new(
        Arc::new(MemoryWalletSessionStore::default()),
        Arc::new(MemoryWalletStore::default()),
    ));
    let assets = Arc::new(GemAssetsService::new(
        api.clone(),
        Arc::new(GemGateway::new(provider.clone(), Arc::new(EmptyPreferences), Arc::new(EmptyPreferences))),
        Arc::new(MemoryAssetStore::default()),
        Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
        Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))),
        wallet_session.clone(),
    ));
    GemWalletConnectService::new(
        Arc::new(GemSimulationService::new(provider, Arc::new(EmptyPreferences))),
        store,
        Arc::new(TestWalletConnectSigner {
            result: signer,
            transactions: Mutex::new(Vec::new()),
        }),
        wallet_session,
        assets,
    )
}
