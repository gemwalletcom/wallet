pub mod error;
pub mod model;
pub mod onboarding;
pub mod password;
mod rules;
pub mod store;

use std::sync::Arc;

use gem_keystore::Mnemonic;
use primitives::{Account, Chain, Wallet, WalletId, WalletSource, WalletType};

use crate::keystore::decode_password;
use crate::keystore::{GemImportType, GemKeystore, GemWalletImport, GemWalletType, keystore_id_for_wallet};
use crate::services::error::GemServiceError;
use crate::services::file::GemFileStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

pub use error::GemWalletImportError;
pub use model::{GemWalletDeletion, GemWalletImportResult, GemWalletImportType, GemWalletSource};
pub use password::{GemKeystoreAuthentication, GemKeystorePassword};
pub use store::GemWalletStore;

const SETUP_CHAINS_WALLETS_LIMIT: usize = 25;

#[derive(uniffi::Object)]
pub struct GemWalletService {
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    store: Arc<dyn GemWalletStore>,
    session: Arc<GemWalletSessionService>,
    app_preferences: Arc<GemPreferencesService>,
    files: Arc<dyn GemFileStore>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemWalletService {
    #[uniffi::constructor]
    pub fn new(
        keystore: Arc<GemKeystore>,
        password: Arc<dyn GemKeystorePassword>,
        store: Arc<dyn GemWalletStore>,
        session: Arc<GemWalletSessionService>,
        app_preferences: Arc<GemPreferencesService>,
        files: Arc<dyn GemFileStore>,
        preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            keystore,
            password,
            store,
            session,
            app_preferences,
            files,
            preferences,
        }
    }

    pub fn create_wallet(&self) -> Result<Vec<String>, GemServiceError> {
        Mnemonic::generate(12).map_err(|error| GemServiceError::Core { msg: error.to_string() })
    }

    pub fn next_wallet_index(&self) -> Result<i32, GemServiceError> {
        Ok(rules::next_wallet_index(&self.store.get_wallets()?))
    }

    pub async fn import_wallet(&self, name: String, import: GemWalletImportType, source: GemWalletSource) -> Result<GemWalletImportResult, GemServiceError> {
        let import = import.validated()?;
        let preview = self.preview_import(import.clone())?;
        let wallet_id = WalletId::from_id(&preview.wallet_id).ok_or_else(|| GemServiceError::Core {
            msg: format!("invalid wallet id {}", preview.wallet_id),
        })?;
        let wallets = self.store.get_wallets()?;
        if let Some(wallet) = rules::existing_wallet(&wallets, &wallet_id, preview.wallet_type) {
            return Ok(GemWalletImportResult::Existing { wallet });
        }
        let index = rules::next_wallet_index(&wallets);
        let wallet = match import {
            GemWalletImportType::Address { address, chain } => Wallet {
                index,
                ..rules::view_wallet(name, chain, address)
            },
            import => {
                let password = decode_password(&self.password.get_password(!self.keystore.has_stored_wallets()?)?);
                let stored = self.keystore.create_store(keystore_import(import), password)?;
                Wallet {
                    id: wallet_id,
                    external_id: None,
                    name,
                    index,
                    wallet_type: stored.wallet_type,
                    accounts: stored.accounts.into_iter().map(rules::account).collect(),
                    is_pinned: false,
                    image_url: None,
                    source,
                }
            }
        };
        self.store.add_wallet(wallet.clone()).await?;
        if wallet.source == WalletSource::Create {
            self.preferences.complete_initial_synchronization(wallet.id.clone())?;
        }
        self.invalidate_subscriptions().await?;
        Ok(GemWalletImportResult::New { wallet })
    }

    pub async fn delete_wallet(&self, wallet_id: WalletId) -> Result<GemWalletDeletion, GemServiceError> {
        let wallet = self.store.get_wallet(wallet_id.clone())?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })?;
        if wallet.wallet_type != WalletType::View {
            self.keystore.delete_wallet_secrets(wallet.id.id(), rules::legacy_keystore_id(&wallet))?;
        }
        self.store.delete_wallet(wallet.id.clone()).await?;
        if let Some(image_url) = wallet.image_url.clone() {
            self.files.remove(image_url)?;
        }
        self.preferences.delete_preferences(wallet.id.clone())?;
        let remaining = self.store.get_wallets()?;
        if remaining.is_empty() || self.session.get_current_wallet_id()? == Some(wallet.id) {
            self.session.set_current_wallet_id(rules::next_current_wallet(&remaining))?;
        }
        if remaining.is_empty() {
            self.app_preferences.clear()?;
        }
        self.invalidate_subscriptions().await?;
        Ok(match remaining.is_empty() {
            true => GemWalletDeletion::LastWalletDeleted,
            false => GemWalletDeletion::WalletsRemaining,
        })
    }

    pub async fn setup_chains(&self, chains: Vec<Chain>) -> Result<Vec<Wallet>, GemServiceError> {
        let candidates: Vec<(Wallet, Vec<Chain>)> = rules::wallets_missing_chains(self.store.get_wallets()?, &chains)
            .into_iter()
            .filter(|(wallet, _)| self.keystore.exists(keystore_id_for_wallet(wallet.id.id())))
            .take(SETUP_CHAINS_WALLETS_LIMIT)
            .collect();
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let password = decode_password(&self.password.get_password(false)?);
        let mut updated = Vec::new();
        for (mut wallet, missing) in candidates {
            let accounts = self.keystore.add_accounts(keystore_id_for_wallet(wallet.id.id()), password.clone(), missing)?;
            wallet.accounts.extend(accounts.into_iter().map(rules::account));
            self.store.add_wallet(wallet.clone()).await?;
            updated.push(wallet);
        }
        if !updated.is_empty() {
            self.invalidate_subscriptions().await?;
        }
        Ok(updated)
    }

    pub fn migrate_to_shared_password(&self) -> Result<u32, GemServiceError> {
        let legacy: Vec<(Wallet, String)> = self
            .store
            .get_wallets()?
            .into_iter()
            .filter(|wallet| wallet.wallet_type != WalletType::View)
            .filter_map(|wallet| match self.password.get_wallet_password(wallet.id.clone()) {
                Ok(Some(password)) => Some(Ok((wallet, password))),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<_, _>>()?;
        if legacy.is_empty() {
            return Ok(0);
        }
        let shared = self.password.get_password(true)?;
        let shared_bytes = decode_password(&shared);
        let mut migrated = 0;
        for (wallet, password) in legacy {
            let keystore_id = keystore_id_for_wallet(wallet.id.id());
            if !self.keystore.exists(keystore_id.clone()) {
                continue;
            }
            if password != shared && !self.keystore.opens_with(keystore_id.clone(), shared_bytes.clone()) {
                self.keystore.change_password(keystore_id.clone(), decode_password(&password), shared_bytes.clone())?;
                if !self.keystore.opens_with(keystore_id.clone(), shared_bytes.clone()) {
                    return Err(GemServiceError::Core {
                        msg: format!("keystore {} did not accept the shared password", wallet.id.id()),
                    });
                }
                migrated += 1;
            }
            self.password.delete_wallet_password(wallet.id.clone())?;
        }
        Ok(migrated)
    }

    pub async fn set_pinned(&self, wallet_id: WalletId, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(wallet_id, pinned).await
    }

    pub async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError> {
        self.store.set_name(wallet_id, name).await
    }

    pub fn sorted_wallets(&self, wallets: Vec<Wallet>) -> Vec<Wallet> {
        rules::sorted_wallets(wallets)
    }

    pub fn display_account(&self, wallet: Wallet) -> Option<Account> {
        rules::display_account(&wallet)
    }
}

impl GemWalletService {
    pub fn preview_import(&self, import: GemWalletImportType) -> Result<GemWalletImport, GemServiceError> {
        match import.validated()? {
            GemWalletImportType::Address { address, chain } => {
                let wallet = rules::view_wallet(String::new(), chain, address);
                Ok(GemWalletImport {
                    wallet_id: wallet.id.id(),
                    wallet_type: GemWalletType::View,
                    accounts: wallet.accounts.into_iter().map(Into::into).collect(),
                })
            }
            import => Ok(self.keystore.preview_import(keystore_import(import))?),
        }
    }

    pub fn wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.store.get_wallets()
    }

    async fn invalidate_subscriptions(&self) -> Result<(), GemServiceError> {
        self.app_preferences.set_subscriptions_version(self.app_preferences.get_subscriptions_version() + 1)
    }
}

fn keystore_import(import: GemWalletImportType) -> GemImportType {
    match import {
        GemWalletImportType::MulticoinPhrase { words, chains } => GemImportType::MulticoinPhrase { words, chains },
        GemWalletImportType::SinglePhrase { words, chain } => GemImportType::SinglePhrase { words, chain },
        GemWalletImportType::PrivateKey { value, chain } => GemImportType::PrivateKey { value, chain },
        GemWalletImportType::Address { address, chain } => GemImportType::PrivateKey { value: address, chain },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use futures::executor::block_on;
    use tempfile::TempDir;

    use super::*;
    use crate::services::file::GemFileStore;
    use crate::services::preferences::GemPreferencesStore;
    use crate::services::wallet_preferences::GemWalletPreferencesStore;
    use crate::services::wallet_session::GemWalletSessionStore;

    const PASSWORD: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const PHRASE: [&str; 12] = [
        "shoot", "island", "position", "soft", "burden", "budget", "tooth", "cruel", "issue", "economy", "destroy", "above",
    ];
    const OTHER_PHRASE: [&str; 12] = [
        "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "about",
    ];

    #[derive(Default)]
    struct MemoryStore {
        wallets: Mutex<Vec<Wallet>>,
        passwords: Mutex<HashMap<String, String>>,
        session: Mutex<Option<WalletId>>,
        preferences: Mutex<HashMap<String, String>>,
        wallet_preferences: Mutex<HashMap<(String, String), String>>,
    }

    #[async_trait::async_trait]
    impl GemWalletStore for MemoryStore {
        fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
            Ok(self.wallets.lock().unwrap().clone())
        }
        fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError> {
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

    impl GemKeystorePassword for MemoryStore {
        fn get_password(&self, _create_if_missing: bool) -> Result<String, GemServiceError> {
            Ok(PASSWORD.to_string())
        }
        fn get_wallet_password(&self, wallet_id: WalletId) -> Result<Option<String>, GemServiceError> {
            Ok(self.passwords.lock().unwrap().get(&wallet_id.id()).cloned())
        }
        fn delete_wallet_password(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
            self.passwords.lock().unwrap().remove(&wallet_id.id());
            Ok(())
        }
        fn authentication(&self) -> Result<GemKeystoreAuthentication, GemServiceError> {
            Ok(GemKeystoreAuthentication::None)
        }
    }

    impl GemWalletSessionStore for MemoryStore {
        fn get_current_wallet_id(&self) -> Result<Option<WalletId>, GemServiceError> {
            Ok(self.session.lock().unwrap().clone())
        }
        fn set_current_wallet_id(&self, wallet_id: Option<WalletId>) -> Result<(), GemServiceError> {
            *self.session.lock().unwrap() = wallet_id;
            Ok(())
        }
    }

    impl GemPreferencesStore for MemoryStore {
        fn get(&self, key: String) -> Option<String> {
            self.preferences.lock().unwrap().get(&key).cloned()
        }
        fn set(&self, key: String, value: String) -> Result<(), GemServiceError> {
            self.preferences.lock().unwrap().insert(key, value);
            Ok(())
        }
        fn remove(&self, key: String) -> Result<(), GemServiceError> {
            self.preferences.lock().unwrap().remove(&key);
            Ok(())
        }
        fn clear(&self) -> Result<(), GemServiceError> {
            self.preferences.lock().unwrap().clear();
            Ok(())
        }
    }

    impl GemWalletPreferencesStore for MemoryStore {
        fn get(&self, wallet_id: WalletId, key: String) -> Option<String> {
            self.wallet_preferences.lock().unwrap().get(&(wallet_id.id(), key)).cloned()
        }
        fn set(&self, wallet_id: WalletId, key: String, value: String) -> Result<(), GemServiceError> {
            self.wallet_preferences.lock().unwrap().insert((wallet_id.id(), key), value);
            Ok(())
        }
        fn delete_preferences(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
            self.wallet_preferences.lock().unwrap().retain(|(id, _), _| *id != wallet_id.id());
            Ok(())
        }
    }

    impl GemFileStore for MemoryStore {
        fn save_file(&self, _data: Vec<u8>, _extension: String) -> Result<String, GemServiceError> {
            Ok(String::new())
        }
        fn save_named_file(&self, _data: Vec<u8>, _file_name: String) -> Result<String, GemServiceError> {
            Ok(String::new())
        }
        fn exists(&self, _file_name: String) -> bool {
            false
        }
        fn path(&self, file_name: String) -> String {
            file_name
        }
        fn remove(&self, _file_name: String) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    struct TestContext {
        service: GemWalletService,
        store: Arc<MemoryStore>,
        directory: TempDir,
    }

    impl TestContext {
        fn new() -> Self {
            let directory = TempDir::new().unwrap();
            let store = Arc::new(MemoryStore::default());
            let keystore = GemKeystore::new(directory.path().to_string_lossy().to_string()).unwrap();
            let session = Arc::new(GemWalletSessionService::new(store.clone(), store.clone()));
            let service = GemWalletService::new(
                keystore,
                store.clone(),
                store.clone(),
                session,
                Arc::new(GemPreferencesService::new(store.clone())),
                store.clone(),
                Arc::new(GemWalletPreferencesService::new(store.clone())),
            );
            Self { service, store, directory }
        }

        async fn import(&self, name: &str, words: [&str; 12]) -> Wallet {
            let import = GemWalletImportType::MulticoinPhrase {
                words: words.iter().map(|word| word.to_string()).collect(),
                chains: vec![Chain::Ethereum],
            };
            match self.service.import_wallet(name.to_string(), import, WalletSource::Import).await.unwrap() {
                GemWalletImportResult::New { wallet } => wallet,
                GemWalletImportResult::Existing { wallet } => wallet,
            }
        }

        fn keystore_path(&self, wallet: &Wallet) -> PathBuf {
            self.directory.path().join(format!("{}.json", keystore_id_for_wallet(wallet.id.id())))
        }
    }

    #[test]
    fn test_delete_wallet_removes_every_secret_copy_and_reports_the_outcome() {
        block_on(async {
            let context = TestContext::new();
            let kept = context.import("Kept", OTHER_PHRASE).await;
            let deleted = context.import("Deleted", PHRASE).await;
            let legacy_path = context.directory.path().join(rules::legacy_keystore_id(&deleted));
            fs::write(&legacy_path, "{}").unwrap();
            context.service.session.set_current_wallet_id(Some(deleted.id.clone())).unwrap();

            let outcome = context.service.delete_wallet(deleted.id.clone()).await.unwrap();

            assert_eq!(outcome, GemWalletDeletion::WalletsRemaining);
            assert!(!context.keystore_path(&deleted).exists());
            assert!(!legacy_path.exists());
            assert!(context.keystore_path(&kept).exists());
            assert_eq!(context.service.session.get_current_wallet_id().unwrap(), Some(kept.id.clone()));

            context.store.preferences.lock().unwrap().insert("is_developer_enabled".to_string(), "true".to_string());

            let outcome = context.service.delete_wallet(kept.id.clone()).await.unwrap();

            assert_eq!(outcome, GemWalletDeletion::LastWalletDeleted);
            assert!(!context.keystore_path(&kept).exists());
            assert_eq!(context.service.session.get_current_wallet_id().unwrap(), None);
            assert_eq!(context.store.preferences.lock().unwrap().get("is_developer_enabled"), None);
        });
    }

    #[test]
    fn test_every_wallet_change_bumps_the_subscriptions_version() {
        block_on(async {
            let context = TestContext::new();
            context.service.app_preferences.set_subscriptions_version(4).unwrap();

            let wallet = context.import("Imported", PHRASE).await;
            assert_eq!(context.service.app_preferences.get_subscriptions_version(), 5, "import must bump");

            context.service.setup_chains(vec![Chain::Ethereum, Chain::Solana]).await.unwrap();
            assert_eq!(context.service.app_preferences.get_subscriptions_version(), 6, "adding a chain must bump");

            let before = context.service.app_preferences.get_subscriptions_version();
            context.service.setup_chains(vec![Chain::Ethereum, Chain::Solana]).await.unwrap();
            assert_eq!(
                context.service.app_preferences.get_subscriptions_version(),
                before,
                "a setup that adds no chain must not bump"
            );

            let second = context.import("Second", OTHER_PHRASE).await;
            let before = context.service.app_preferences.get_subscriptions_version();
            context.service.delete_wallet(second.id.clone()).await.unwrap();
            assert_eq!(
                context.service.app_preferences.get_subscriptions_version(),
                before + 1,
                "deleting one of several wallets must bump"
            );

            context.service.delete_wallet(wallet.id.clone()).await.unwrap();
            assert_eq!(
                context.service.app_preferences.get_subscriptions_version(),
                1,
                "deleting the last wallet clears preferences first, so the bump restarts from zero"
            );
        });
    }

    #[test]
    fn test_migration_rekeys_a_legacy_wallet_and_can_be_run_again() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Legacy", PHRASE).await;
            let keystore_id = keystore_id_for_wallet(wallet.id.id());
            let legacy = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            context
                .service
                .keystore
                .change_password(keystore_id.clone(), decode_password(PASSWORD), decode_password(legacy))
                .unwrap();
            context.store.passwords.lock().unwrap().insert(wallet.id.id(), legacy.to_string());

            assert_eq!(context.service.migrate_to_shared_password().unwrap(), 1);
            assert!(context.service.keystore.opens_with(keystore_id.clone(), decode_password(PASSWORD)));
            assert!(context.store.passwords.lock().unwrap().is_empty());

            assert_eq!(context.service.migrate_to_shared_password().unwrap(), 0);
            assert!(context.service.keystore.opens_with(keystore_id, decode_password(PASSWORD)));
        });
    }

    #[test]
    fn test_migration_drops_an_alias_that_already_holds_the_shared_password() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Aliased", PHRASE).await;
            context.store.passwords.lock().unwrap().insert(wallet.id.id(), PASSWORD.to_string());

            assert_eq!(context.service.migrate_to_shared_password().unwrap(), 0);
            assert!(context.store.passwords.lock().unwrap().is_empty());
            assert!(context.service.keystore.opens_with(keystore_id_for_wallet(wallet.id.id()), decode_password(PASSWORD)));
        });
    }

    #[test]
    fn test_migration_keeps_the_legacy_password_when_rekeying_fails() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Unmigratable", PHRASE).await;
            let keystore_id = keystore_id_for_wallet(wallet.id.id());
            let actual = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            let wrong = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
            context
                .service
                .keystore
                .change_password(keystore_id.clone(), decode_password(PASSWORD), decode_password(actual))
                .unwrap();
            context.store.passwords.lock().unwrap().insert(wallet.id.id(), wrong.to_string());

            assert!(context.service.migrate_to_shared_password().is_err());
            assert_eq!(context.store.passwords.lock().unwrap().get(&wallet.id.id()).map(String::as_str), Some(wrong));
            assert!(context.service.keystore.opens_with(keystore_id, decode_password(actual)));
        });
    }

    #[test]
    fn test_migration_keeps_the_password_of_a_wallet_with_no_v4_keystore() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("PendingV3", PHRASE).await;
            let keystore_id = keystore_id_for_wallet(wallet.id.id());
            let legacy = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            context.store.passwords.lock().unwrap().insert(wallet.id.id(), legacy.to_string());
            context.service.keystore.delete(keystore_id.clone()).unwrap();

            assert_eq!(context.service.migrate_to_shared_password().unwrap(), 0);
            assert_eq!(context.store.passwords.lock().unwrap().get(&wallet.id.id()).map(String::as_str), Some(legacy));
        });
    }

    #[test]
    fn test_migration_clears_a_legacy_entry_left_by_an_interrupted_run() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Interrupted", PHRASE).await;
            let stale = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            context.store.passwords.lock().unwrap().insert(wallet.id.id(), stale.to_string());

            assert_eq!(context.service.migrate_to_shared_password().unwrap(), 0);
            assert!(context.store.passwords.lock().unwrap().is_empty());
            assert!(context.service.keystore.opens_with(keystore_id_for_wallet(wallet.id.id()), decode_password(PASSWORD)));
        });
    }

    #[test]
    fn test_delete_wallet_keeps_the_record_when_a_secret_copy_survives() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Blocked", PHRASE).await;
            fs::remove_file(context.keystore_path(&wallet)).unwrap();
            fs::create_dir(context.keystore_path(&wallet)).unwrap();

            let error = context.service.delete_wallet(wallet.id.clone()).await.unwrap_err();

            assert!(matches!(error, GemServiceError::Core { .. }), "{error:?}");
            assert_eq!(context.store.get_wallets().unwrap().len(), 1);
        });
    }
}
