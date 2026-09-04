pub mod error;
pub mod model;
pub mod password;
pub mod rules;
pub mod store;
impl GemWalletService {
    fn migrate_wallet_password(&self, wallet: &Wallet, password: &str, shared: &str) -> Result<bool, GemServiceError> {
        let keystore_id = keystore_id_for_wallet(wallet.id.id());
        if !self.keystore.exists(keystore_id.clone()) {
            return Ok(false);
        }
        let shared_bytes = decode_password(shared);
        let rekeyed = password != shared && !self.keystore.opens_with(keystore_id.clone(), shared_bytes.clone());
        if rekeyed {
            self.keystore.change_password(keystore_id.clone(), decode_password(password), shared_bytes.clone())?;
            if !self.keystore.opens_with(keystore_id, shared_bytes) {
                return Err(GemServiceError::Core {
                    msg: "did not accept the shared password".to_string(),
                });
            }
        }
        self.password.delete_wallet_password(wallet.id.clone())?;
        Ok(rekeyed)
    }
}

#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use gem_keystore::Mnemonic;
use primitives::{Chain, Wallet, WalletId, WalletSource, WalletType};

use crate::block_explorer::GemBlockExplorerLink;
use crate::keystore::decode_password;
use crate::keystore::{GemImportType, GemKeystore, GemWalletImport, keystore_id_for_wallet};
use crate::services::error::GemServiceError;
use crate::services::localization::{GemLocalizedText, GemLocalizer};
use crate::services::explorer::GemExplorerService;
use crate::services::file::GemFileStore;
use crate::services::name::GemAddressStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

pub use error::GemWalletImportError;
pub use model::{GemWalletDeletion, GemWalletImportResult, GemWalletDefaultName, GemWalletImportType, GemWalletSecret};
pub use password::{GemKeystoreAuthentication, GemKeystorePassword};
pub use store::GemWalletStore;

const SETUP_CHAINS_WALLETS_LIMIT: usize = 25;

#[derive(Default)]
pub struct SetupChainsOutcome {
    pub wallets: Vec<Wallet>,
    pub failures: Vec<(WalletId, GemServiceError)>,
}

#[derive(uniffi::Object)]
pub struct GemWalletService {
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    store: Arc<dyn GemWalletStore>,
    session: Arc<GemWalletSessionService>,
    app_preferences: Arc<GemPreferencesService>,
    files: Arc<dyn GemFileStore>,
    preferences: Arc<GemWalletPreferencesService>,
    explorer: Arc<GemExplorerService>,
    addresses: Arc<dyn GemAddressStore>,
    localizer: Arc<dyn GemLocalizer>,
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
        explorer: Arc<GemExplorerService>,
        addresses: Arc<dyn GemAddressStore>,
        localizer: Arc<dyn GemLocalizer>,
    ) -> Self {
        Self {
            keystore,
            password,
            store,
            session,
            app_preferences,
            files,
            preferences,
            explorer,
            addresses,
            localizer,
        }
    }

    pub fn current_wallet_id(&self) -> Result<Option<WalletId>, GemServiceError> {
        self.session.get_current_wallet_id()
    }

    pub fn set_current_wallet_id(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.session.set_current_wallet_id(Some(wallet_id))
    }

    pub fn address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub fn create_wallet(&self) -> Result<Vec<String>, GemServiceError> {
        Mnemonic::generate(12).map_err(|error| GemServiceError::Core { msg: error.to_string() })
    }

    pub async fn default_wallet_name(&self, chain: Option<Chain>) -> Result<GemWalletDefaultName, GemServiceError> {
        let index = rules::next_wallet_index(&self.store.get_wallets().await?);
        let text = match chain {
            Some(chain) => GemLocalizedText::WalletDefaultNameChain { chain, index },
            None => GemLocalizedText::WalletDefaultName { index },
        };
        Ok(GemWalletDefaultName {
            name: self.localizer.text(text),
            has_existing_wallets: index > 1,
        })
    }



    pub async fn import_wallet(&self, name: String, import: GemWalletImportType, source: WalletSource) -> Result<GemWalletImportResult, GemServiceError> {
        let import = import.validated()?;
        let preview = self.preview_import(import.clone())?;
        let wallet_id = WalletId::from_id(&preview.wallet_id).ok_or_else(|| GemServiceError::Core {
            msg: format!("invalid wallet id {}", preview.wallet_id),
        })?;
        let wallets = self.store.get_wallets().await?;
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
        self.store_wallet(&wallet).await?;
        if wallet.source == WalletSource::Create {
            self.preferences.complete_initial_synchronization(wallet.id.clone())?;
        }
        self.invalidate_subscriptions().await?;
        Ok(GemWalletImportResult::New { wallet })
    }

    pub async fn delete_wallet(&self, wallet_id: WalletId) -> Result<GemWalletDeletion, GemServiceError> {
        let wallet = self.store.get_wallet(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })?;
        if wallet.wallet_type != WalletType::View {
            self.keystore.delete_wallet_secrets(wallet.id.id(), rules::legacy_keystore_id(&wallet))?;
        }
        self.store.delete_wallet(wallet.id.clone()).await?;
        self.addresses.delete_address_names(rules::wallet_address_names(&wallet)).await?;
        if let Some(image_url) = wallet.image_url.clone() {
            self.files.remove(image_url)?;
        }
        self.preferences.delete_preferences(wallet.id.clone())?;
        let remaining = self.store.get_wallets().await?;
        if remaining.is_empty() || self.session.get_current_wallet_id()? == Some(wallet.id) {
            self.session.set_current_wallet_id(rules::next_current_wallet(&remaining))?;
        }
        self.invalidate_subscriptions().await?;
        Ok(match remaining.is_empty() {
            true => GemWalletDeletion::LastWalletDeleted,
            false => GemWalletDeletion::WalletsRemaining,
        })
    }

    pub async fn export_secret(&self, wallet_id: WalletId) -> Result<GemWalletSecret, GemServiceError> {
        let wallet = self.store.get_wallet(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })?;
        let keystore_id = keystore_id_for_wallet(wallet.id.id());
        let password = decode_password(&self.password.get_password(false)?);
        match rules::secret_export(&wallet) {
            rules::SecretExport::Words => Ok(GemWalletSecret::Words {
                words: self.keystore.export_recovery_phrase(keystore_id, password)?,
            }),
            rules::SecretExport::PrivateKey(chain) => Ok(GemWalletSecret::PrivateKey {
                key: self.keystore.export_private_key(keystore_id, chain, password)?,
            }),
            rules::SecretExport::None => Err(GemServiceError::Core {
                msg: format!("wallet {} keeps no secret", wallet.id.id()),
            }),
        }
    }

    pub async fn setup_chains(&self, chains: Vec<Chain>) -> Result<Vec<Wallet>, GemServiceError> {
        let SetupChainsOutcome { wallets, failures } = self.setup_chains_outcome(chains).await?;
        match failures.into_iter().next() {
            Some((_, error)) if wallets.is_empty() => Err(error),
            _ => Ok(wallets),
        }
    }

    pub async fn migrate_to_shared_password(&self) -> Result<u32, GemServiceError> {
        let legacy: Vec<(Wallet, String)> = self
            .store
            .get_wallets().await?
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
        let mut migrated = 0;
        let mut failures = Vec::new();
        for (wallet, password) in legacy {
            match self.migrate_wallet_password(&wallet, &password, &shared) {
                Ok(true) => migrated += 1,
                Ok(false) => {}
                Err(error) => failures.push(format!("keystore {}: {error}", wallet.id.id())),
            }
        }
        if failures.is_empty() {
            return Ok(migrated);
        }
        Err(GemServiceError::Core { msg: failures.join("; ") })
    }

    pub async fn set_pinned(&self, wallet_id: WalletId, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(wallet_id, pinned).await
    }

    pub async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError> {
        let wallet = self.store.get_wallet(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })?;
        self.store.set_name(wallet_id, name.clone()).await?;
        self.addresses.save_address_names(rules::wallet_address_names(&Wallet { name, ..wallet })).await
    }

    pub fn sorted_wallets(&self, wallets: Vec<Wallet>) -> Vec<Wallet> {
        rules::sorted_wallets(wallets)
    }

    pub async fn wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.store.get_wallets().await
    }
}

impl GemWalletService {
    pub fn preview_import(&self, import: GemWalletImportType) -> Result<GemWalletImport, GemServiceError> {
        match import.validated()? {
            GemWalletImportType::Address { address, chain } => {
                let wallet = rules::view_wallet(String::new(), chain, address);
                Ok(GemWalletImport {
                    wallet_id: wallet.id.id(),
                    wallet_type: WalletType::View,
                    accounts: wallet.accounts.into_iter().map(Into::into).collect(),
                })
            }
            import => Ok(self.keystore.preview_import(keystore_import(import))?),
        }
    }

    pub async fn setup_chains_outcome(&self, chains: Vec<Chain>) -> Result<SetupChainsOutcome, GemServiceError> {
        let candidates: Vec<(Wallet, Vec<Chain>)> = rules::wallets_missing_chains(self.store.get_wallets().await?, &chains)
            .into_iter()
            .filter(|(wallet, _)| self.keystore.exists(keystore_id_for_wallet(wallet.id.id())))
            .take(SETUP_CHAINS_WALLETS_LIMIT)
            .collect();
        if candidates.is_empty() {
            return Ok(SetupChainsOutcome::default());
        }
        let password = decode_password(&self.password.get_password(false)?);
        let mut outcome = SetupChainsOutcome::default();
        for (mut wallet, missing) in candidates {
            match self.add_chains(&mut wallet, missing, password.clone()).await {
                Ok(()) => outcome.wallets.push(wallet),
                Err(error) => outcome.failures.push((wallet.id, error)),
            }
        }
        if !outcome.wallets.is_empty() {
            self.invalidate_subscriptions().await?;
        }
        Ok(outcome)
    }

    async fn add_chains(&self, wallet: &mut Wallet, missing: Vec<Chain>, password: Vec<u8>) -> Result<(), GemServiceError> {
        let accounts = self.keystore.add_accounts(keystore_id_for_wallet(wallet.id.id()), password, missing)?;
        wallet.accounts.extend(accounts.into_iter().map(rules::account));
        self.store_wallet(wallet).await
    }

    async fn store_wallet(&self, wallet: &Wallet) -> Result<(), GemServiceError> {
        self.store.add_wallet(wallet.clone()).await?;
        self.addresses.save_address_names(rules::wallet_address_names(wallet)).await
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
    use std::fs;
    use std::path::PathBuf;

    use futures::executor::block_on;
    use primitives::{AddressType, Currency};
    use tempfile::TempDir;

    use super::testkit::{MemoryAddressStore, MemoryKeystorePassword, MemoryWalletStore, TEST_PASSWORD};
    use super::*;
    use crate::services::localization::testkit::EnglishLocalizer;
    use crate::services::file::testkit::NoopFileStore;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
    use crate::services::wallet_session::testkit::MemoryWalletSessionStore;

    const PHRASE: [&str; 12] = [
        "shoot", "island", "position", "soft", "burden", "budget", "tooth", "cruel", "issue", "economy", "destroy", "above",
    ];
    const OTHER_PHRASE: [&str; 12] = [
        "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "abandon", "about",
    ];

    struct TestContext {
        service: GemWalletService,
        passwords: Arc<MemoryKeystorePassword>,
        addresses: Arc<MemoryAddressStore>,
        directory: TempDir,
    }

    impl TestContext {
        fn new() -> Self {
            let directory = TempDir::new().unwrap();
            let wallets = Arc::new(MemoryWalletStore::default());
            let passwords = Arc::new(MemoryKeystorePassword::default());
            let addresses = Arc::new(MemoryAddressStore::default());
            let preferences = Arc::new(MemoryPreferencesStore::default());
            let keystore = GemKeystore::new(directory.path().to_string_lossy().to_string()).unwrap();
            let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets.clone()));
            let app_preferences = Arc::new(GemPreferencesService::new(preferences.clone()));
            let service = GemWalletService::new(
                keystore,
                passwords.clone(),
                wallets,
                session,
                app_preferences.clone(),
                Arc::new(NoopFileStore),
                Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default()))),
                Arc::new(GemExplorerService::new(app_preferences)),
                addresses.clone(),
                Arc::new(EnglishLocalizer),
            );
            Self {
                service,
                passwords,
                addresses,
                directory,
            }
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

        fn lock_out(&self, wallet: &Wallet) {
            let password = decode_password(&self.service.password.get_password(false).unwrap());
            self.service
                .keystore
                .change_password(keystore_id_for_wallet(wallet.id.id()), password, b"other".to_vec())
                .unwrap();
        }
    }

    #[test]
    fn test_delete_wallet_removes_secret_copies_preserves_preferences_and_reports_outcome() {
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

            context.service.app_preferences.set_currency(Currency::EUR).unwrap();

            let outcome = context.service.delete_wallet(kept.id.clone()).await.unwrap();

            assert_eq!(outcome, GemWalletDeletion::LastWalletDeleted);
            assert!(!context.keystore_path(&kept).exists());
            assert_eq!(context.service.session.get_current_wallet_id().unwrap(), None);
            assert_eq!(context.service.app_preferences.get_currency(), Currency::EUR);
        });
    }

    #[test]
    fn test_wallet_accounts_are_named_after_the_wallet_until_it_is_deleted() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Savings", PHRASE).await;
            let account = wallet.accounts[0].clone();
            let name = async |context: &TestContext| context.addresses.get_address_name(account.chain, account.address.clone()).await.unwrap();

            let stored = name(&context).await.unwrap();
            assert_eq!((stored.name.as_str(), stored.address_type), ("Savings", AddressType::InternalWallet));

            context.service.rename(wallet.id.clone(), "Spending".to_string()).await.unwrap();
            assert_eq!(name(&context).await.unwrap().name, "Spending");

            context.service.delete_wallet(wallet.id.clone()).await.unwrap();
            assert!(name(&context).await.is_none());
        });
    }

    #[test]
    fn test_import_creates_the_keystore_password_only_while_the_keystore_is_empty() {
        block_on(async {
            let context = TestContext::new();
            context.import("First", PHRASE).await;
            context.import("Second", OTHER_PHRASE).await;

            assert_eq!(*context.passwords.create_requests.lock().unwrap(), vec![true, false]);
        });
    }

    #[test]
    fn test_export_secret_follows_the_wallet_type() {
        block_on(async {
            let context = TestContext::new();
            let phrase = context.import("Phrase", PHRASE).await;
            let key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".to_string();
            let import = GemWalletImportType::PrivateKey {
                value: key.clone(),
                chain: Chain::Ethereum,
            };
            let GemWalletImportResult::New { wallet: private } = context.service.import_wallet("Key".to_string(), import, WalletSource::Import).await.unwrap() else {
                panic!("expected a new wallet");
            };
            let view = rules::view_wallet("View".to_string(), Chain::Ethereum, private.accounts[0].address.clone());
            context.service.store.add_wallet(view.clone()).await.unwrap();

            assert_eq!(
                context.service.export_secret(phrase.id.clone()).await.unwrap(),
                GemWalletSecret::Words {
                    words: PHRASE.iter().map(|word| word.to_string()).collect()
                }
            );
            assert_eq!(context.service.export_secret(private.id.clone()).await.unwrap(), GemWalletSecret::PrivateKey { key });
            assert!(context.service.export_secret(view.id.clone()).await.is_err());
        });
    }

    #[test]
    fn test_setup_chains_keeps_going_when_one_keystore_cannot_be_read() {
        block_on(async {
            let context = TestContext::new();
            let broken = context.import("Broken", PHRASE).await;
            let healthy = context.import("Healthy", OTHER_PHRASE).await;
            context.lock_out(&broken);

            let outcome = context.service.setup_chains_outcome(vec![Chain::Ethereum, Chain::Solana]).await.unwrap();

            assert_eq!(outcome.failures.len(), 1);
            assert_eq!(outcome.failures[0].0, broken.id);
            assert_eq!(outcome.wallets.len(), 1, "the healthy wallet must still be set up");
            assert_eq!(outcome.wallets[0].id, healthy.id);
            let stored = context.service.wallets().await.unwrap();
            let stored_healthy = stored.iter().find(|wallet| wallet.id == healthy.id).unwrap();
            assert!(stored_healthy.accounts.iter().any(|account| account.chain == Chain::Solana));
        });
    }

    #[test]
    fn test_setup_chains_reports_the_error_when_no_wallet_could_be_set_up() {
        block_on(async {
            let context = TestContext::new();
            let only = context.import("Only", PHRASE).await;
            context.lock_out(&only);

            let error = context.service.setup_chains(vec![Chain::Ethereum, Chain::Solana]).await;

            assert!(error.is_err(), "a setup that added nothing must surface the failure");
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

            let before = context.service.app_preferences.get_subscriptions_version();
            context.service.delete_wallet(wallet.id.clone()).await.unwrap();
            assert_eq!(
                context.service.app_preferences.get_subscriptions_version(),
                before + 1,
                "deleting the last wallet must bump without resetting app preferences"
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
                .change_password(keystore_id.clone(), decode_password(TEST_PASSWORD), decode_password(legacy))
                .unwrap();
            context.passwords.wallet_passwords.lock().unwrap().insert(wallet.id.id(), legacy.to_string());

            assert_eq!(context.service.migrate_to_shared_password().await.unwrap(), 1);
            assert!(context.service.keystore.opens_with(keystore_id.clone(), decode_password(TEST_PASSWORD)));
            assert!(context.passwords.wallet_passwords.lock().unwrap().is_empty());

            assert_eq!(context.service.migrate_to_shared_password().await.unwrap(), 0);
            assert!(context.service.keystore.opens_with(keystore_id, decode_password(TEST_PASSWORD)));
        });
    }

    #[test]
    fn test_migration_drops_an_alias_that_already_holds_the_shared_password() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Aliased", PHRASE).await;
            context.passwords.wallet_passwords.lock().unwrap().insert(wallet.id.id(), TEST_PASSWORD.to_string());

            assert_eq!(context.service.migrate_to_shared_password().await.unwrap(), 0);
            assert!(context.passwords.wallet_passwords.lock().unwrap().is_empty());
            assert!(context.service.keystore.opens_with(keystore_id_for_wallet(wallet.id.id()), decode_password(TEST_PASSWORD)));
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
                .change_password(keystore_id.clone(), decode_password(TEST_PASSWORD), decode_password(actual))
                .unwrap();
            context.passwords.wallet_passwords.lock().unwrap().insert(wallet.id.id(), wrong.to_string());
            let other = context.import("Migratable", OTHER_PHRASE).await;
            let other_keystore_id = keystore_id_for_wallet(other.id.id());
            context
                .service
                .keystore
                .change_password(other_keystore_id.clone(), decode_password(TEST_PASSWORD), decode_password(actual))
                .unwrap();
            context.passwords.wallet_passwords.lock().unwrap().insert(other.id.id(), actual.to_string());

            assert!(context.service.migrate_to_shared_password().await.is_err());
            assert_eq!(context.passwords.wallet_passwords.lock().unwrap().get(&wallet.id.id()).map(String::as_str), Some(wrong));
            assert!(context.service.keystore.opens_with(keystore_id, decode_password(actual)));
            assert!(
                context.service.keystore.opens_with(other_keystore_id, decode_password(TEST_PASSWORD)),
                "one wallet that cannot be re-keyed does not stop the others from migrating"
            );
            assert_eq!(context.passwords.wallet_passwords.lock().unwrap().get(&other.id.id()), None);
        });
    }

    #[test]
    fn test_migration_keeps_the_password_of_a_wallet_with_no_v4_keystore() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("PendingV3", PHRASE).await;
            let keystore_id = keystore_id_for_wallet(wallet.id.id());
            let legacy = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            context.passwords.wallet_passwords.lock().unwrap().insert(wallet.id.id(), legacy.to_string());
            context.service.keystore.delete(keystore_id.clone()).unwrap();

            assert_eq!(context.service.migrate_to_shared_password().await.unwrap(), 0);
            assert_eq!(context.passwords.wallet_passwords.lock().unwrap().get(&wallet.id.id()).map(String::as_str), Some(legacy));
        });
    }

    #[test]
    fn test_migration_clears_a_legacy_entry_left_by_an_interrupted_run() {
        block_on(async {
            let context = TestContext::new();
            let wallet = context.import("Interrupted", PHRASE).await;
            let stale = "0f0e0d0c0b0a09080706050403020100f0e0d0c0b0a090807060504030201000";
            context.passwords.wallet_passwords.lock().unwrap().insert(wallet.id.id(), stale.to_string());

            assert_eq!(context.service.migrate_to_shared_password().await.unwrap(), 0);
            assert!(context.passwords.wallet_passwords.lock().unwrap().is_empty());
            assert!(context.service.keystore.opens_with(keystore_id_for_wallet(wallet.id.id()), decode_password(TEST_PASSWORD)));
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
            assert_eq!(context.service.wallets().await.unwrap().len(), 1);
        });
    }
}
