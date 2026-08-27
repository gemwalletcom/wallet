pub mod rules;

use std::sync::Arc;

use primitives::{AuthNonce, AuthPayload, Wallet, hex};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::auth::create_auth_message;
use crate::device::device_public_key;
use crate::keystore::{GemKeystore, keystore_id_for_wallet};
use crate::services::error::GemServiceError;
use crate::services::wallet::GemKeystorePassword;

#[derive(uniffi::Object)]
pub struct GemAuthService {
    api: Arc<GemDeviceApiClient>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    device_private_key: Vec<u8>,
}

#[uniffi::export]
impl GemAuthService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, keystore: Arc<GemKeystore>, password: Arc<dyn GemKeystorePassword>, device_private_key: Vec<u8>) -> Self {
        Self {
            api,
            keystore,
            password,
            device_private_key,
        }
    }
}

impl GemAuthService {
    pub async fn get_nonce(&self) -> Result<AuthNonce, GemServiceError> {
        Ok(self.api.client.get_auth_nonce().await.map_err(GemApiError::from)?)
    }

    pub async fn get_auth_payload(&self, wallet: Wallet) -> Result<AuthPayload, GemServiceError> {
        let account = rules::auth_account(&wallet).ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} has no {} account", wallet.id.id(), rules::AUTH_CHAIN),
        })?;
        let nonce = self.get_nonce().await?;
        let message = create_auth_message(&account.address, nonce.clone());
        let password = self.password.get_password(wallet.id.clone(), false)?;
        let signature = self.keystore.sign_auth(keystore_id_for_wallet(wallet.id.id()), rules::AUTH_CHAIN, message.hash, password)?;
        Ok(AuthPayload {
            device_id: hex::encode(device_public_key(self.device_private_key.clone())?),
            chain: rules::AUTH_CHAIN,
            address: account.address.clone(),
            nonce: nonce.nonce,
            signature,
        })
    }
}
