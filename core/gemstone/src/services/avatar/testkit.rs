use std::sync::Arc;

use super::GemAvatarService;
use crate::services::file::GemFileStore;
use crate::services::wallet::GemWalletStore;
use crate::testkit::TestAlienProvider;

impl GemAvatarService {
    pub fn mock(wallets: Arc<dyn GemWalletStore>, files: Arc<dyn GemFileStore>) -> Self {
        Self::new(wallets, files, Arc::new(TestAlienProvider::with_status(200)))
    }
}
