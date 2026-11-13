use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use gem_spark::{LIGHTNING_SCHEME, LightningWallet, decode_invoice};
use primitives::time::unix_timestamp;
use primitives::{AddressFormatStyle, Asset, AssetId, Chain, Wallet, WalletId};

use super::model::{GemLightningReceive, GemLightningReceiveState};
use super::runtime::run;
use super::sessions;
use crate::address_formatter::format_address;
use crate::api::GemApiClient;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::models::copy::{GemCopy, GemCopyKind};
use crate::services::amount::model::{GemAmountField, GemAmountKeyboard, GemAmountSymbol, GemAmountSymbolPlacement, GemNumberFormat};
use crate::services::amount::rules::value_from_input;
use crate::services::balance::GemBalanceService;
use crate::services::error::{GemServiceError, required_account};
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet::GemKeystorePassword;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemLightningService {
    api: Arc<GemApiClient>,
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
    pub fn new(
        api: Arc<GemApiClient>,
        keystore: Arc<GemKeystore>,
        password: Arc<dyn GemKeystorePassword>,
        storage_dir: String,
        balances: Arc<GemBalanceService>,
        transactions: Arc<GemTransactionsService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            api,
            keystore,
            password,
            storage_dir: PathBuf::from(storage_dir),
            balances,
            transactions,
            session,
        }
    }

    pub async fn open_enabled(&self) -> Result<(), GemServiceError> {
        let generation = sessions().generation();
        let Some(wallet) = self.session.get_current_wallet().await? else {
            return Ok(());
        };
        let Some(account) = wallet.account(Chain::Lightning) else {
            return Ok(());
        };
        if !self.balances.is_asset_enabled(wallet.id.clone(), AssetId::from_chain(Chain::Lightning)).await? {
            return Ok(());
        }
        let (lightning, opened) = self.open_wallet(generation, wallet.id.clone(), &account.address).await?;
        if opened {
            let _ = run(async move { lightning.sync().await }).await;
            let _ = self.update_wallet(wallet.id).await;
        }
        self.refresh_partner_token().await;
        Ok(())
    }

    pub async fn receive(&self, wallet_id: WalletId, amount: String, format: GemNumberFormat) -> Result<GemLightningReceive, GemServiceError> {
        let wallet = self.wallet(wallet_id).await?;
        let asset = Asset::from_chain(Chain::Lightning);
        let amount_sats = value_from_input(&format.decimal_separator, &amount, asset.decimals).ok().and_then(|value| u64::try_from(value).ok()).filter(|value| *value > 0);
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

    fn wallet_storage_dir(&self, wallet_id: &WalletId) -> PathBuf {
        self.storage_dir.join(keystore_id_for_wallet(wallet_id.id()))
    }

    async fn wallet(&self, wallet_id: WalletId) -> Result<Arc<LightningWallet>, GemServiceError> {
        let generation = sessions().generation();
        let address = required_account(&self.session.require_wallet(wallet_id.clone()).await?, Chain::Lightning)?.address;
        Ok(self.open_wallet(generation, wallet_id, &address).await?.0)
    }

    async fn open_wallet(&self, generation: u64, wallet_id: WalletId, address: &str) -> Result<(Arc<LightningWallet>, bool), GemServiceError> {
        sessions()
            .open(generation, address, || async {
                let password = decode_password(&self.password.get_password(false)?);
                let account = self.keystore.lightning_account_key(&keystore_id_for_wallet(wallet_id.id()), password)?;
                let storage_dir = self.wallet_storage_dir(&wallet_id);
                let partner = sessions().partner();
                let (_, wallet) = futures::join!(self.refresh_partner_token(), run(async move { LightningWallet::open(account, &storage_dir, partner).await }));
                Ok(wallet?)
            })
            .await
    }

    async fn refresh_partner_token(&self) {
        let partner = sessions().partner();
        if !partner.needs_token() {
            return;
        }
        if let Ok(token) = self.api.client.get_lightning_token().await {
            partner.set(token);
        }
    }

    async fn update_wallet(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let lightning = vec![AssetId::from_chain(Chain::Lightning)];
        let (balances, transactions) = futures::join!(self.balances.update(wallet_id.clone(), lightning), self.transactions.sync_lightning(wallet_id));
        balances?;
        transactions
    }
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
    use gem_spark::testkit::INVOICE_2000_SATS;

    #[test]
    fn test_receive() {
        let receive = receive(INVOICE_2000_SATS.to_string(), Asset::from_chain(Chain::Lightning));

        assert_eq!(receive.qr, format!("lightning:{INVOICE_2000_SATS}"));
        assert_eq!(receive.copy.value, INVOICE_2000_SATS);
        assert!(receive.copy.display.len() < INVOICE_2000_SATS.len());
        assert_eq!(receive.field.symbol, GemAmountSymbol::Asset { symbol: "BTC".to_string() });
    }

    #[test]
    fn test_receive_state() {
        assert_eq!(receive_state(true, 100, 200), GemLightningReceiveState::Received);
        assert_eq!(receive_state(false, 100, 100), GemLightningReceiveState::Expired);
        assert_eq!(receive_state(false, 100, 99), GemLightningReceiveState::Pending);
    }
}
