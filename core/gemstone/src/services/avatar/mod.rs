pub mod store;

use std::sync::Arc;

use primitives::WalletId;

use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::services::error::GemServiceError;
use crate::services::wallet::GemWalletStore;
pub use store::GemFileStore;

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
        let file_name = self.files.save(image, IMAGE_EXTENSION.to_string())?;
        self.remove_previous(&wallet_id)?;
        self.wallets.set_image_url(wallet_id, Some(file_name)).await
    }

    pub async fn set_image_url(&self, wallet_id: WalletId, url: String) -> Result<(), GemServiceError> {
        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };
        let response = self
            .provider
            .request(target)
            .await
            .map_err(|error| GemServiceError::Api { msg: error.to_string() })?
            .to_rpc_response();
        if let Some(status) = response.status
            && !(200..300).contains(&status)
        {
            return Err(GemServiceError::Api {
                msg: format!("image download failed with status {status}"),
            });
        }
        self.set_image(wallet_id, response.data).await
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
