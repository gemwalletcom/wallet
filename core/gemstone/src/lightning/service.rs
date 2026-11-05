use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use gem_spark::LightningWallet;
use primitives::{AssetId, Chain, Wallet, WalletId};

use super::wallet::run;
use super::{GemLightningWallet, sessions};
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet::GemKeystorePassword;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemLightningService {
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    storage_dir: PathBuf,
    balances: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemLightningService {
    #[uniffi::constructor]
    pub fn new(keystore: Arc<GemKeystore>, password: Arc<dyn GemKeystorePassword>, storage_dir: String, balances: Arc<GemBalanceService>, transactions: Arc<GemTransactionsService>, session: Arc<GemWalletSessionService>) -> Self {
        Self {
            keystore,
            password,
            storage_dir: PathBuf::from(storage_dir),
            balances,
            transactions,
            session,
        }
    }

    pub async fn open_enabled(&self, wallet_id: WalletId) -> Result<bool, GemServiceError> {
        let Some(address) = self.address(wallet_id.clone()).await? else {
            return Ok(false);
        };
        if !self.balances.is_asset_enabled(wallet_id.clone(), AssetId::from_chain(Chain::Lightning)).await? {
            return Ok(false);
        }
        let wallet = self.open_wallet(wallet_id.clone(), &address).await?;
        let _ = self.sync_wallet(wallet_id, wallet).await;
        Ok(true)
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let Some(address) = self.address(wallet_id.clone()).await? else {
            return Ok(());
        };
        let Ok(wallet) = sessions().wallet(&address) else {
            return Ok(());
        };
        self.sync_wallet(wallet_id, wallet).await
    }

    pub async fn open(&self, wallet_id: WalletId) -> Result<Arc<GemLightningWallet>, GemServiceError> {
        let address = self.address(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::core("wallet has no lightning account"))?;
        Ok(GemLightningWallet::new(self.open_wallet(wallet_id, &address).await?))
    }

    pub fn lock_all(&self) {
        sessions().lock_all();
    }
}

impl GemLightningService {
    pub(crate) async fn delete_wallet(&self, wallet: &Wallet, delete_secrets: impl FnOnce() -> Result<(), GemServiceError>) -> Result<(), GemServiceError> {
        let address = wallet.account(Chain::Lightning).map(|account| account.address.as_str());
        let storage_dir = self.wallet_storage_dir(&wallet.id);
        sessions()
            .close_and_remove(address, || {
                match fs::remove_dir_all(storage_dir) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(GemServiceError::core(error)),
                }
                delete_secrets()
            })
            .await
    }

    async fn address(&self, wallet_id: WalletId) -> Result<Option<String>, GemServiceError> {
        Ok(self.session.get_wallet(wallet_id).await?.and_then(|wallet| wallet.account(Chain::Lightning).map(|account| account.address.clone())))
    }

    fn wallet_storage_dir(&self, wallet_id: &WalletId) -> PathBuf {
        self.storage_dir.join(keystore_id_for_wallet(wallet_id.id()))
    }

    async fn open_wallet(&self, wallet_id: WalletId, address: &str) -> Result<Arc<LightningWallet>, GemServiceError> {
        sessions()
            .open(address, || async {
                let password = decode_password(&self.password.get_password(false)?);
                Ok(self.keystore.lightning_wallet(&keystore_id_for_wallet(wallet_id.id()), password, self.wallet_storage_dir(&wallet_id)).await?)
            })
            .await
    }

    async fn sync_wallet(&self, wallet_id: WalletId, wallet: Arc<LightningWallet>) -> Result<(), GemServiceError> {
        let synced = run(async move { wallet.sync().await }).await;
        let lightning = vec![AssetId::from_chain(Chain::Lightning)];
        let (balances, transactions) = futures::join!(self.balances.update(wallet_id.clone(), lightning), self.transactions.sync_lightning(wallet_id));
        balances?;
        transactions?;
        Ok(synced?)
    }
}
