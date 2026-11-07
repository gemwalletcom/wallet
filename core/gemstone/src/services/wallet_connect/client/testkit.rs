use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use gem_encoding::decode_base64;
use primitives::{Platform, Wallet, WalletConnectionVerificationStatus};

use crate::services::wallet_connect::rules;
use serde_json::Value;
use wallet_connect_protocol::crypto::{self, Key};
use wallet_connect_protocol::envelope;
use wallet_connect_protocol::sign::Metadata;
use wallet_connect_protocol::sign::method::METHOD_SESSION_PROPOSE;
use wallet_connect_protocol::testkit::{TEST_PEER_PUBLIC_KEY, TEST_PEER_SECRET, TEST_SYM_KEY, TEST_TOPIC, test_key};

use super::state::{AnsweredRequest, Outgoing, Pairing, PendingRequest, Session};
use super::{GemWalletConnectClient, GemWalletConnectEvent, GemWalletConnectListener, GemWalletConnectProposal, GemWalletConnectTransport};
use crate::alien::AlienProvider;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemSecureStore;
use crate::services::preferences::testkit::MemorySecureStore;
use crate::services::wallet_connect::GemWalletConnectService;
use crate::testkit::TestAlienProvider;

pub const TEST_VERIFIED_ORIGIN: &str = r#"{"origin":"https://example.com","isScam":false}"#;
pub const TEST_PROPOSAL: &str = include_str!("../../../../../crates/wallet_connect_protocol/testdata/session_proposal.json");
pub const TEST_AUTHENTICATION: &str = include_str!("../../../../../crates/wallet_connect_protocol/testdata/authenticate_request.json");

#[derive(Default)]
pub struct MemoryTransport {
    pub frames: Mutex<Vec<String>>,
    pub stalled: AtomicBool,
}

#[async_trait]
impl GemWalletConnectTransport for MemoryTransport {
    async fn send(&self, message: String) -> Result<(), GemServiceError> {
        if self.stalled.load(Ordering::SeqCst) {
            return futures::future::pending().await;
        }
        self.frames.lock().unwrap().push(message);
        Ok(())
    }
}

impl MemoryTransport {
    pub fn sent(&self) -> Vec<Value> {
        self.frames.lock().unwrap().iter().map(|frame| serde_json::from_str(frame).unwrap()).collect()
    }

    pub fn published(&self, topic: &str, key: &Key) -> Vec<Value> {
        self.sent()
            .into_iter()
            .filter(|frame| frame["method"] == "irn_publish" && frame["params"]["topic"] == topic)
            .map(|frame| serde_json::from_slice(&envelope::decrypt(key, frame["params"]["message"].as_str().unwrap()).unwrap()).unwrap())
            .collect()
    }

    pub fn published_with_sender_key(&self, topic: &str) -> Vec<Value> {
        self.sent()
            .into_iter()
            .filter(|frame| frame["method"] == "irn_publish" && frame["params"]["topic"] == topic)
            .map(|frame| {
                let envelope = decode_base64(frame["params"]["message"].as_str().unwrap()).unwrap();
                let sender: [u8; 32] = envelope[1..33].try_into().unwrap();
                let key = crypto::shared_key(&test_key(TEST_PEER_SECRET), &sender).unwrap();
                serde_json::from_slice(&crypto::open(&key, &envelope[33..]).unwrap()).unwrap()
            })
            .collect()
    }
}

#[derive(Default)]
pub struct MemoryListener {
    pub events: Mutex<Vec<GemWalletConnectEvent>>,
}

impl GemWalletConnectListener for MemoryListener {
    fn on_event(&self, event: GemWalletConnectEvent) {
        self.events.lock().unwrap().push(event);
    }
}

pub struct TestDapp {
    pub transport: Arc<MemoryTransport>,
    pub listener: Arc<MemoryListener>,
    pub store: Arc<MemorySecureStore>,
}

impl TestDapp {
    pub fn pairing_uri(methods: &str) -> String {
        format!("wc:{TEST_TOPIC}@2?relay-protocol=irn&symKey={TEST_SYM_KEY}&methods={methods}")
    }

    pub fn frame(topic: &str, key: &Key, payload: &str) -> String {
        let message = envelope::encrypt(key, payload.as_bytes()).unwrap();
        format!(r#"{{"id":1,"jsonrpc":"2.0","method":"irn_subscription","params":{{"id":"subscription","data":{{"topic":"{topic}","message":"{message}","publishedAt":1}}}}}}"#)
    }

    pub fn reply(id: u64) -> String {
        format!(r#"{{"id":{id},"jsonrpc":"2.0","result":true}}"#)
    }

    pub fn request(id: u64, method: &str, params: &str) -> String {
        format!(r#"{{"id":{id},"jsonrpc":"2.0","method":"{method}","params":{params}}}"#)
    }

    pub fn session_key(responder_public_key: &str) -> Key {
        crypto::shared_key(&test_key(TEST_PEER_SECRET), &crypto::parse_key(responder_public_key).unwrap()).unwrap()
    }

    pub fn topic(key: &Key) -> String {
        crypto::topic(key.as_slice())
    }

    pub fn relaunched(&self) -> Self {
        Self {
            transport: Arc::new(MemoryTransport::default()),
            listener: Arc::new(MemoryListener::default()),
            store: self.store.clone(),
        }
    }

    pub fn requester_topic() -> String {
        crypto::topic(&crypto::public_key(&test_key(TEST_PEER_SECRET)))
    }
}

impl GemWalletConnectClient {
    pub async fn mock(dapp: &TestDapp, platform: Platform) -> Self {
        Self::mock_with_service(dapp, platform, GemWalletConnectService::mock(Ok("0xsignature".to_string()), Wallet::mock()).await)
    }

    pub fn mock_with_service(dapp: &TestDapp, platform: Platform, service: GemWalletConnectService) -> Self {
        let provider: Arc<dyn AlienProvider> = Arc::new(TestAlienProvider::with_json_by_path(200, &[("verify.walletconnect.org", TEST_VERIFIED_ORIGIN)]));
        let store: Arc<dyn GemSecureStore> = dapp.store.clone();
        Self::new(Arc::new(service), store, provider, dapp.transport.clone(), dapp.listener.clone(), platform)
    }
}

impl TestDapp {
    pub fn new() -> Self {
        Self {
            transport: Arc::new(MemoryTransport::default()),
            listener: Arc::new(MemoryListener::default()),
            store: Arc::new(MemorySecureStore::default()),
        }
    }
}

impl Pairing {
    pub fn mock(topic: &str, expiry: u64) -> Self {
        Self {
            topic: topic.to_string(),
            key: TEST_SYM_KEY.to_string(),
            expiry,
            authenticates: false,
        }
    }
}

impl Session {
    pub fn mock(topic: &str, expiry: u64) -> Self {
        Self {
            topic: topic.to_string(),
            key: TEST_SYM_KEY.to_string(),
            peer: Metadata::mock(),
            namespaces: BTreeMap::new(),
            expiry,
            origin: None,
            validation: WalletConnectionVerificationStatus::Unknown,
        }
    }
}

impl PendingRequest {
    pub fn mock(topic: &str, id: u64, expiry: Option<u64>) -> Self {
        Self {
            topic: topic.to_string(),
            id,
            method: "personal_sign".to_string(),
            params: "[]".to_string(),
            chain_id: "eip155:1".to_string(),
            expiry,
            shown: false,
        }
    }
}

impl GemWalletConnectClient {
    pub async fn mock_connected(dapp: &TestDapp, wallet: Wallet) -> (Self, Key) {
        let service = GemWalletConnectService::mock(Ok("0xsignature".to_string()), wallet.clone()).await;
        let client = Self::mock_with_service(dapp, Platform::IOS, service);
        let pairing_key = test_key(TEST_SYM_KEY);
        client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
        client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL))).await.unwrap();
        client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet).await.unwrap();
        let approval = dapp.transport.sent().into_iter().find(|frame| frame["method"] == "wc_approveSession").unwrap();
        let response: Value = serde_json::from_slice(&envelope::decrypt(&pairing_key, approval["params"]["sessionProposalResponse"].as_str().unwrap()).unwrap()).unwrap();
        (client, TestDapp::session_key(response["result"]["responderPublicKey"].as_str().unwrap()))
    }
}

impl GemWalletConnectProposal {
    pub fn mock() -> Self {
        Self {
            id: TEST_PEER_PUBLIC_KEY.to_string(),
            pairing_topic: TEST_TOPIC.to_string(),
            metadata: rules::application_metadata("Example".to_string(), "Example dapp".to_string(), "https://example.com".to_string(), vec!["https://example.com/icon.png".to_string()]),
            required_chain_ids: vec!["eip155:1".to_string()],
            optional_chain_ids: vec!["eip155:137".to_string(), "eip155:56".to_string(), "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string()],
            origin: Some("https://example.com".to_string()),
            validation: WalletConnectionVerificationStatus::Verified,
        }
    }
}

impl AnsweredRequest {
    pub fn mock(id: u64, expiry: u64) -> Self {
        Self { topic: "topic".to_string(), id, expiry }
    }
}

impl Outgoing {
    pub fn mock(id: u64, expiry: u64) -> Self {
        Self { id, frame: "{}".to_string(), expiry }
    }
}
