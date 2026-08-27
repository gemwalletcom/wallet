use async_trait::async_trait;
use primitives::{WalletConnection, WalletConnectionSession};

use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemConnectionStore: Send + Sync {
    async fn get_connection(&self, session_id: String) -> Result<Option<WalletConnection>, GemServiceError>;
    async fn get_sessions(&self) -> Result<Vec<WalletConnectionSession>, GemServiceError>;
    async fn add_connection(&self, connection: WalletConnection) -> Result<(), GemServiceError>;
    async fn update_session(&self, session: WalletConnectionSession) -> Result<(), GemServiceError>;
    async fn delete_sessions(&self, session_ids: Vec<String>) -> Result<(), GemServiceError>;
}
