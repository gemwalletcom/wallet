use std::sync::Arc;

use gem_spark::{AccountKey, LIGHTNING_SCHEME, LightningWallet, decode_invoice};
use primitives::time::unix_timestamp;
use primitives::{AddressFormatStyle, Asset, AssetId, Chain, WalletId};

use super::model::{GemLightningReceive, GemLightningReceiveState};
use super::runtime::run;
use super::sessions::SESSIONS;
use super::store::WalletLightningStore;
use crate::address_formatter::format_address;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::models::copy::{GemCopy, GemCopyKind};
use crate::services::amount::model::{GemAmountField, GemAmountKeyboard, GemAmountSymbol, GemAmountSymbolPlacement, GemNumberFormat};
use crate::services::amount::rules::value_from_input;
use crate::services::balance::GemBalanceService;
use crate::services::error::{GemServiceError, required_account};
use crate::services::preferences::GemSecureStore;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet::GemKeystorePassword;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemLightningService {
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    secure: Arc<dyn GemSecureStore>,
    preferences: Arc<GemWalletPreferencesService>,
    balances: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemLightningService {
    #[uniffi::constructor]
    pub fn new(
        keystore: Arc<GemKeystore>,
        password: Arc<dyn GemKeystorePassword>,
        secure: Arc<dyn GemSecureStore>,
        preferences: Arc<GemWalletPreferencesService>,
        balances: Arc<GemBalanceService>,
        transactions: Arc<GemTransactionsService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            keystore,
            password,
            secure,
            preferences,
            balances,
            transactions,
            session,
        }
    }

    pub async fn open_enabled(&self) -> Result<(), GemServiceError> {
        let Some(wallet) = self.session.get_current_wallet().await? else {
            return Ok(());
        };
        let Some(account) = wallet.account(Chain::Lightning) else {
            return Ok(());
        };
        if !self.balances.is_asset_enabled(wallet.id.clone(), AssetId::from_chain(Chain::Lightning)).await? {
            return Ok(());
        }
        let (lightning, opened) = self.open_wallet(wallet.id.clone(), &account.address).await?;
        if opened {
            let _ = run(async move { lightning.sync().await }).await;
            let _ = self.update_wallet(wallet.id).await;
        }
        Ok(())
    }

    pub async fn receive(&self, wallet_id: WalletId, amount: String, format: GemNumberFormat) -> Result<GemLightningReceive, GemServiceError> {
        let wallet = self.wallet(wallet_id).await?;
        let asset = Asset::from_chain(Chain::Lightning);
        let amount_sats = value_from_input(&format.decimal_separator, &amount, asset.decimals).ok().and_then(|value| u64::try_from(value).ok()).unwrap_or_default();
        let invoice = run(async move { wallet.create_invoice(amount_sats).await }).await?;
        Ok(receive(invoice, asset))
    }

    pub async fn receive_state(&self, wallet_id: WalletId, invoice: String) -> Result<GemLightningReceiveState, GemServiceError> {
        let wallet = self.wallet(wallet_id.clone()).await?;
        let expires_at = decode_invoice(&invoice)?.expires_at;
        let received = run(async move {
            wallet.sync().await?;
            Ok(wallet.payments().await?.iter().any(|payment| payment.receives(&invoice)))
        })
        .await?;
        if received {
            self.update_wallet(wallet_id).await?;
        }
        Ok(receive_state(received, expires_at, unix_timestamp()))
    }
}

impl GemLightningService {
    async fn wallet(&self, wallet_id: WalletId) -> Result<Arc<LightningWallet>, GemServiceError> {
        let address = required_account(&self.session.require_wallet(wallet_id.clone()).await?, Chain::Lightning)?.address;
        Ok(self.open_wallet(wallet_id, &address).await?.0)
    }

    async fn open_wallet(&self, wallet_id: WalletId, address: &str) -> Result<(Arc<LightningWallet>, bool), GemServiceError> {
        SESSIONS
            .open(address, || async {
                let account = self.account_key(&keystore_id_for_wallet(wallet_id.id()))?;
                let store = Arc::new(WalletLightningStore::new(self.preferences.clone(), wallet_id.clone()));
                Ok(run(async move { LightningWallet::open(account, store).await }).await?)
            })
            .await
    }

    fn account_key(&self, keystore_id: &str) -> Result<AccountKey, GemServiceError> {
        let key = account_key_name(keystore_id);
        if let Some(value) = self.secure.get(key.clone())? {
            return Ok(AccountKey::decode(&value)?);
        }
        let account = self.keystore.lightning_account_key(keystore_id, decode_password(&self.password.get_password(false)?))?;
        self.secure.set(key, account.encode())?;
        Ok(account)
    }

    async fn update_wallet(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let lightning = vec![AssetId::from_chain(Chain::Lightning)];
        let (balances, transactions) = futures::join!(self.balances.update(wallet_id.clone(), lightning), self.transactions.sync_lightning(wallet_id));
        balances?;
        transactions
    }
}

pub(crate) fn account_key_name(keystore_id: &str) -> String {
    format!("lightning_account_{keystore_id}")
}

fn receive(invoice: String, asset: Asset) -> GemLightningReceive {
    GemLightningReceive {
        qr: format!("{LIGHTNING_SCHEME}{invoice}"),
        copy: GemCopy {
            kind: GemCopyKind::Plain,
            display: format_address(&invoice, Some(Chain::Lightning), AddressFormatStyle::Short),
            value: invoice,
        },
        field: GemAmountField {
            symbol: GemAmountSymbol::Asset { symbol: asset.symbol },
            placement: GemAmountSymbolPlacement::Trailing,
            keyboard: GemAmountKeyboard::Decimal,
        },
    }
}

fn receive_state(received: bool, expires_at: u64, now: u64) -> GemLightningReceiveState {
    match received {
        true => GemLightningReceiveState::Received,
        false if expires_at <= now => GemLightningReceiveState::Expired,
        false => GemLightningReceiveState::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_state() {
        assert_eq!(receive_state(true, 100, 200), GemLightningReceiveState::Received);
        assert_eq!(receive_state(false, 100, 100), GemLightningReceiveState::Expired);
        assert_eq!(receive_state(false, 100, 99), GemLightningReceiveState::Pending);
    }
}
