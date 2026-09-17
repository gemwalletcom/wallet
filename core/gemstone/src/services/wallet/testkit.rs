use std::collections::HashMap;
use std::sync::Mutex;

use primitives::{AddressName, Chain, Wallet, WalletId};

use super::GemWalletStore;
use super::password::{GemKeystoreAuthentication, GemKeystorePassword};
use crate::services::error::GemServiceError;
use crate::services::name::GemAddressStore;
use std::path::PathBuf;
use std::sync::Arc;

use primitives::WalletSource;
use tempfile::TempDir;

use super::{GemWalletImportResult, GemWalletImportType, GemWalletService, keystore_id_for_wallet};
use crate::keystore::GemKeystore;
use crate::keystore::decode_password;
use crate::services::avatar::GemAvatarService;
use crate::services::explorer::GemExplorerService;
use crate::services::file::testkit::NoopFileStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::TestAlienProvider;

pub const TEST_PASSWORD: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[derive(Default)]
pub struct MemoryWalletStore {
    pub wallets: Mutex<Vec<Wallet>>,
    pub add_wallet_error: Mutex<Option<GemServiceError>>,
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
        if let Some(error) = self.add_wallet_error.lock().unwrap().clone() {
            return Err(error);
        }
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
    pub save_error: Mutex<Option<GemServiceError>>,
}

#[async_trait::async_trait]
impl GemAddressStore for MemoryAddressStore {
    async fn get_address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError> {
        Ok(self.names.lock().unwrap().get(&(chain, address)).cloned())
    }
    async fn save_address_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError> {
        if let Some(error) = self.save_error.lock().unwrap().clone() {
            return Err(error);
        }
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

pub const PHRASE: [&str; 12] = [
    "shoot", "island", "position", "soft", "burden", "budget", "tooth", "cruel", "issue", "economy", "destroy", "above",
];
pub const OTHER_PHRASE: [&str; 12] = [
    "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "about",
];

pub struct WalletTestkit {
    pub service: Arc<GemWalletService>,
    pub wallets: Arc<MemoryWalletStore>,
    pub passwords: Arc<MemoryKeystorePassword>,
    pub addresses: Arc<MemoryAddressStore>,
    pub keystore: Arc<GemKeystore>,
    pub directory: TempDir,
}

impl WalletTestkit {
    pub fn new() -> Self {
        let directory = TempDir::new().unwrap();
        let wallets = Arc::new(MemoryWalletStore::default());
        let passwords = Arc::new(MemoryKeystorePassword::default());
        let addresses = Arc::new(MemoryAddressStore::default());
        let preferences = Arc::new(MemoryPreferencesStore::default());
        let keystore = GemKeystore::new(directory.path().to_string_lossy().to_string()).unwrap();
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets.clone()));
        let app_preferences = Arc::new(GemPreferencesService::new(preferences.clone()));
        let service = Arc::new(GemWalletService::new(
            keystore.clone(),
            passwords.clone(),
            wallets.clone(),
            session,
            app_preferences.clone(),
            Arc::new(NoopFileStore),
            Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default()))),
            Arc::new(GemExplorerService::new(app_preferences)),
            addresses.clone(),
            Arc::new(GemAvatarService::new(
                wallets.clone(),
                Arc::new(NoopFileStore),
                Arc::new(TestAlienProvider::new(crate::alien::AlienResponse::new(None, Vec::new()))),
            )),
        ));
        Self {
            service,
            wallets,
            passwords,
            addresses,
            keystore,
            directory,
        }
    }

    pub async fn import(&self, name: &str, words: [&str; 12]) -> Wallet {
        let import = GemWalletImportType::MulticoinPhrase {
            words: words.iter().map(|word| word.to_string()).collect(),
            chains: vec![Chain::Ethereum],
        };
        match self.service.import_wallet(name.to_string(), import, WalletSource::Import).await.unwrap() {
            GemWalletImportResult::New { wallet } => wallet,
            GemWalletImportResult::Existing { wallet } => wallet,
        }
    }

    pub fn keystore_path(&self, wallet: &Wallet) -> PathBuf {
        self.directory.path().join(format!("{}.json", keystore_id_for_wallet(wallet.id.id())))
    }

    pub fn lock_out(&self, wallet: &Wallet) {
        let password = decode_password(&self.service.password.get_password(false).unwrap());
        self.service
            .keystore
            .change_password(keystore_id_for_wallet(wallet.id.id()), password, b"other".to_vec())
            .unwrap();
    }
}
