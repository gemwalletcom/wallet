use std::sync::Arc;

use primitives::WalletId;

use crate::alien::AlienProvider;
use crate::services::error::GemServiceError;
use crate::services::file::{GemFileStore, IMAGE_EXTENSION, download};
use crate::services::wallet::GemWalletStore;

/// Holds `GemWalletStore` rather than `GemWalletService`, which owns it: the wallet service holds
/// this one and forwards the avatar calls, so the reverse edge would be a cycle.
#[derive(uniffi::Object)]
pub struct GemAvatarService {
    wallets: Arc<dyn GemWalletStore>,
    files: Arc<dyn GemFileStore>,
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl GemAvatarService {
    #[uniffi::constructor]
    pub fn new(wallets: Arc<dyn GemWalletStore>, files: Arc<dyn GemFileStore>, provider: Arc<dyn AlienProvider>) -> Self {
        Self { wallets, files, provider }
    }
}

impl GemAvatarService {
    pub async fn set_image(&self, wallet_id: WalletId, image: Vec<u8>) -> Result<(), GemServiceError> {
        let previous = self.current_image(&wallet_id).await?;
        let file_name = self.files.save_file(image, IMAGE_EXTENSION.to_string())?;
        if let Err(error) = self.wallets.set_image_url(wallet_id, Some(file_name.clone())).await {
            let _ = self.files.remove(file_name);
            return Err(error);
        }
        self.retire(previous)
    }

    pub async fn set_image_url(&self, wallet_id: WalletId, url: String) -> Result<(), GemServiceError> {
        let image = download(&self.provider, url).await?;
        self.set_image(wallet_id, image).await
    }

    pub async fn remove_image(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let previous = self.current_image(&wallet_id).await?;
        self.wallets.set_image_url(wallet_id, None).await?;
        self.retire(previous)
    }
}

impl GemAvatarService {
    async fn current_image(&self, wallet_id: &WalletId) -> Result<Option<String>, GemServiceError> {
        Ok(self.wallets.get_wallet(wallet_id.clone()).await?.and_then(|wallet| wallet.image_url))
    }

    fn retire(&self, previous: Option<String>) -> Result<(), GemServiceError> {
        match previous {
            Some(previous) => self.files.remove(previous),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::file::testkit::MemoryFileStore;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use primitives::Wallet;
    use std::sync::Mutex;

    fn service(files: Arc<MemoryFileStore>, wallets: Arc<MemoryWalletStore>) -> GemAvatarService {
        GemAvatarService {
            wallets,
            files,
            provider: Arc::new(TestAlienProvider::with_status(200)),
        }
    }

    fn stored_wallet(image_url: Option<&str>) -> (Arc<MemoryWalletStore>, Wallet) {
        let wallet = Wallet {
            image_url: image_url.map(str::to_string),
            ..Wallet::mock()
        };
        let wallets = Arc::new(MemoryWalletStore {
            wallets: Mutex::new(vec![wallet.clone()]),
            ..Default::default()
        });
        (wallets, wallet)
    }

    #[test]
    fn test_the_avatar_a_wallet_points_at_always_exists() {
        let (wallets, wallet) = stored_wallet(Some("old.png"));
        let files = Arc::new(MemoryFileStore::with_file("old.png"));

        block_on(service(files.clone(), wallets.clone()).set_image(wallet.id.clone(), vec![1])).unwrap();

        let stored = block_on(wallets.get_wallet(wallet.id.clone())).unwrap().unwrap();
        assert_eq!(*files.files.lock().unwrap(), vec![stored.image_url.clone().unwrap()], "the replaced file is retired only once its replacement is published");
        assert_eq!(*files.removed.lock().unwrap(), vec!["old.png".to_string()]);
    }

    #[test]
    fn test_a_failed_write_keeps_the_avatar_the_wallet_still_points_at() {
        let (wallets, wallet) = stored_wallet(Some("old.png"));
        *wallets.set_image_url_error.lock().unwrap() = Some(GemServiceError::Store { msg: "disk full".to_string() });
        let files = Arc::new(MemoryFileStore::with_file("old.png"));

        assert!(block_on(service(files.clone(), wallets.clone()).set_image(wallet.id.clone(), vec![1])).is_err());

        assert_eq!(*files.files.lock().unwrap(), vec!["old.png".to_string()], "the published avatar survives and the unpublished one is not left behind");
        assert_eq!(block_on(wallets.get_wallet(wallet.id)).unwrap().unwrap().image_url, Some("old.png".to_string()));
    }

    #[test]
    fn test_a_failed_removal_leaves_the_avatar_readable() {
        let (wallets, wallet) = stored_wallet(Some("old.png"));
        *wallets.set_image_url_error.lock().unwrap() = Some(GemServiceError::Store { msg: "disk full".to_string() });
        let files = Arc::new(MemoryFileStore::with_file("old.png"));

        assert!(block_on(service(files.clone(), wallets.clone()).remove_image(wallet.id)).is_err());

        assert_eq!(*files.files.lock().unwrap(), vec!["old.png".to_string()], "a wallet that still points at its avatar keeps the file");
    }
}
