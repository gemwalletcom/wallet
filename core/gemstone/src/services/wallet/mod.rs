pub mod error;
pub mod model;
pub mod password;
mod rules;
pub mod store;

use std::sync::Arc;

use gem_keystore::Mnemonic;
use primitives::{Account, Chain, Wallet, WalletId, WalletSource, WalletType};

use crate::keystore::{GemImportType, GemKeystore, GemWalletImport, GemWalletType, keystore_id_for_wallet};
use crate::services::error::GemServiceError;
use crate::services::file::GemFileStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

pub use error::GemWalletImportError;
pub use model::{GemWalletImportResult, GemWalletImportType, GemWalletSource};
pub use password::GemKeystorePassword;
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

    pub fn validate_import(&self, import: GemWalletImportType) -> Result<GemWalletImportType, GemWalletImportError> {
        rules::validate_import(import)
    }

    pub async fn import_wallet(&self, name: String, import: GemWalletImportType, source: GemWalletSource) -> Result<GemWalletImportResult, GemServiceError> {
        let import = rules::validate_import(import)?;
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
                let password = self.password.get_password(wallet_id.clone(), true)?;
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

    pub async fn delete_wallet(&self, wallet_id: WalletId) -> Result<bool, GemServiceError> {
        let wallet = self.store.get_wallet(wallet_id.clone())?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })?;
        if wallet.wallet_type != WalletType::View {
            self.keystore.delete(keystore_id_for_wallet(wallet.id.id()))?;
        }
        self.store.delete_wallet(wallet.id.clone()).await?;
        if let Some(image_url) = wallet.image_url.clone() {
            self.files.remove(image_url)?;
        }
        self.preferences.delete_preferences(wallet.id.clone())?;
        let remaining = self.store.get_wallets()?;
        if self.session.get_current_wallet_id()? == Some(wallet.id) {
            self.session.set_current_wallet_id(rules::next_current_wallet(&remaining))?;
        }
        if remaining.is_empty() {
            self.app_preferences.clear()?;
        }
        self.invalidate_subscriptions().await?;
        Ok(!remaining.is_empty())
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
        let mut updated = Vec::new();
        for (mut wallet, missing) in candidates {
            let password = self.password.get_password(wallet.id.clone(), false)?;
            let accounts = self.keystore.add_accounts(keystore_id_for_wallet(wallet.id.id()), password, missing)?;
            wallet.accounts.extend(accounts.into_iter().map(rules::account));
            self.store.add_wallet(wallet.clone()).await?;
            updated.push(wallet);
        }
        if !updated.is_empty() {
            self.invalidate_subscriptions().await?;
        }
        Ok(updated)
    }

    pub async fn set_pinned(&self, wallet_id: WalletId, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(wallet_id, pinned).await
    }

    pub async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError> {
        self.store.set_name(wallet_id, name).await
    }
}

impl GemWalletService {
    pub fn preview_import(&self, import: GemWalletImportType) -> Result<GemWalletImport, GemServiceError> {
        match rules::validate_import(import)? {
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

#[uniffi::export]
pub fn sorted_wallets(wallets: Vec<Wallet>) -> Vec<Wallet> {
    rules::sorted_wallets(wallets)
}

#[uniffi::export]
pub fn wallet_display_account(wallet: Wallet) -> Option<Account> {
    rules::display_account(&wallet)
}
