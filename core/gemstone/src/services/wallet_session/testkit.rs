use std::sync::Mutex;

use primitives::WalletId;

use super::GemWalletSessionStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryWalletSessionStore {
    pub current: Mutex<Option<WalletId>>,
}

impl GemWalletSessionStore for MemoryWalletSessionStore {
    fn get_current_wallet_id(&self) -> Result<Option<WalletId>, GemServiceError> {
        Ok(self.current.lock().unwrap().clone())
    }
    fn set_current_wallet_id(&self, wallet_id: Option<WalletId>) -> Result<(), GemServiceError> {
        *self.current.lock().unwrap() = wallet_id;
        Ok(())
    }
}
