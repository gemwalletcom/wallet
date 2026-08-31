use std::sync::Arc;

use primitives::WalletId;

use crate::alien::AlienProvider;
use crate::services::error::GemServiceError;
use crate::services::file::{GemFileStore, download};
use crate::services::wallet::GemWalletStore;

const IMAGE_EXTENSION: &str = "png";

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

    pub async fn set_image(&self, wallet_id: WalletId, image: Vec<u8>) -> Result<(), GemServiceError> {
        let file_name = self.files.save_file(image, IMAGE_EXTENSION.to_string())?;
        self.remove_previous(&wallet_id)?;
        self.wallets.set_image_url(wallet_id, Some(file_name)).await
    }

    pub async fn set_image_url(&self, wallet_id: WalletId, url: String) -> Result<(), GemServiceError> {
        let image = download(&self.provider, url).await?;
        self.set_image(wallet_id, image).await
    }

    pub async fn remove_image(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.remove_previous(&wallet_id)?;
        self.wallets.set_image_url(wallet_id, None).await
    }
}

impl GemAvatarService {
    fn remove_previous(&self, wallet_id: &WalletId) -> Result<(), GemServiceError> {
        match self.wallets.get_wallet(wallet_id.clone())?.and_then(|wallet| wallet.image_url) {
            Some(previous) => self.files.remove(previous),
            None => Ok(()),
        }
    }
}
