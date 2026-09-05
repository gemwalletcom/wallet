use std::collections::HashMap;
use std::sync::Mutex;

use primitives::{AddressName, Chain, Wallet, WalletId};

use super::GemWalletStore;
use super::password::{GemKeystoreAuthentication, GemKeystorePassword};
use crate::services::error::GemServiceError;
use crate::services::name::GemAddressStore;

pub const TEST_PASSWORD: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[derive(Default)]
pub struct MemoryWalletStore {
    pub wallets: Mutex<Vec<Wallet>>,
}

#[async_trait::async_trait]
impl GemWalletStore for MemoryWalletStore {
    async fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        Ok(self.wallets.lock().unwrap().clone())
    }
    async fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError> {
        Ok(self.wallets.lock().unwrap().iter().find(|wallet| wallet.id == wallet_id).cloned())
    }
    async fn add_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        let mut wallets = self.wallets.lock().unwrap();
        wallets.retain(|stored| stored.id != wallet.id);
        wallets.push(wallet);
        Ok(())
    }
    async fn delete_wallet(&self, wallet_id: WalletId) -> Result<bool, GemServiceError> {
        let mut wallets = self.wallets.lock().unwrap();
        let before = wallets.len();
        wallets.retain(|wallet| wallet.id != wallet_id);
        Ok(before != wallets.len())
    }
    async fn set_pinned(&self, _wallet_id: WalletId, _pinned: bool) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_name(&self, _wallet_id: WalletId, _name: String) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_image_url(&self, _wallet_id: WalletId, _image_url: Option<String>) -> Result<(), GemServiceError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct MemoryKeystorePassword {
    pub wallet_passwords: Mutex<HashMap<String, String>>,
    pub create_requests: Mutex<Vec<bool>>,
}

impl GemKeystorePassword for MemoryKeystorePassword {
    fn get_password(&self, create_if_missing: bool) -> Result<String, GemServiceError> {
        self.create_requests.lock().unwrap().push(create_if_missing);
        Ok(TEST_PASSWORD.to_string())
    }
    fn get_wallet_password(&self, wallet_id: WalletId) -> Result<Option<String>, GemServiceError> {
        Ok(self.wallet_passwords.lock().unwrap().get(&wallet_id.id()).cloned())
    }
    fn delete_wallet_password(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.wallet_passwords.lock().unwrap().remove(&wallet_id.id());
        Ok(())
    }
    fn authentication(&self) -> Result<GemKeystoreAuthentication, GemServiceError> {
        Ok(GemKeystoreAuthentication::None)
    }
}

#[derive(Default)]
pub struct MemoryAddressStore {
    pub names: Mutex<HashMap<(Chain, String), AddressName>>,
}

#[async_trait::async_trait]
impl GemAddressStore for MemoryAddressStore {
    async fn get_address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError> {
        Ok(self.names.lock().unwrap().get(&(chain, address)).cloned())
    }
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError> {
        let mut stored = self.names.lock().unwrap();
        for name in names {
            stored.insert((name.chain, name.address.clone()), name);
        }
        Ok(())
    }
    async fn delete_address_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError> {
        let mut stored = self.names.lock().unwrap();
        for name in names {
            stored.remove(&(name.chain, name.address));
        }
        Ok(())
    }
}
