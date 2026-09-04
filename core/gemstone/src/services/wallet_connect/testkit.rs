use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{WalletConnection, WalletConnectionSession};

use super::{GemConnectionStore, GemWalletConnectMessageRequest, GemWalletConnectSigner, GemWalletConnectTransactionRequest};
use crate::services::error::GemServiceError;

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
}

#[async_trait]
impl GemWalletConnectSigner for TestWalletConnectSigner {
    async fn sign_message(&self, _request: GemWalletConnectMessageRequest) -> Result<String, GemServiceError> {
        self.result.clone()
    }
    async fn sign_transaction(&self, _request: GemWalletConnectTransactionRequest) -> Result<String, GemServiceError> {
        self.result.clone()
    }
}
