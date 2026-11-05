use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::secp256k1::{Message, PublicKey, Secp256k1};
use spark_sqlite::SqliteTreeStore;
use spark_wallet::{ListTransfersRequest, Network, Order, PagingFilter, SparkSignerAdapter, SparkWallet, SparkWalletConfig, TransferId, WalletBuilder};
use tokio::sync::watch;

use crate::address::SparkAddress;
use crate::error::LightningError;
use crate::invoice::{decode_invoice, requested_amount, send_amount};
use crate::keys::AccountKey;
use crate::mapper::map_transfer;
use crate::model::{LightningInvoice, LightningNetwork, LightningPayment, LightningSendQuote};
use crate::signer::LockableSigner;
use crate::store::{PendingSend, WalletDatabase};

const DATABASE_FILE: &str = "spark.sqlite";
const AMOUNTLESS_INVOICE_SATS: u64 = 0;
const SIGNATURE_LEN: usize = 65;
const COMPACT_SIGNATURE_LEN: usize = 64;

pub struct LightningWallet {
    wallet: Arc<SparkWallet>,
    database: Arc<WalletDatabase>,
    network: LightningNetwork,
    lock: SessionLock,
}

struct SessionLock {
    signer: Arc<LockableSigner>,
    cancel: watch::Sender<()>,
    sends: AtomicUsize,
    requested: AtomicBool,
}

struct SendPermit<'a>(&'a SessionLock);

impl SessionLock {
    fn new(signer: Arc<LockableSigner>, cancel: watch::Sender<()>) -> Self {
        Self {
            signer,
            cancel,
            sends: AtomicUsize::new(0),
            requested: AtomicBool::new(false),
        }
    }

    fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
        if self.sends.load(Ordering::SeqCst) == 0 {
            self.apply();
        }
    }

    fn apply(&self) {
        let _ = self.cancel.send(());
        self.signer.lock();
    }

    fn begin_send(&self) -> Result<SendPermit<'_>, LightningError> {
        self.sends.fetch_add(1, Ordering::SeqCst);
        let permit = SendPermit(self);
        if self.requested.load(Ordering::SeqCst) {
            return Err(LightningError::Locked);
        }
        Ok(permit)
    }
}

impl Drop for SendPermit<'_> {
    fn drop(&mut self) {
        if self.0.sends.fetch_sub(1, Ordering::SeqCst) == 1 && self.0.requested.load(Ordering::SeqCst) {
            self.0.apply();
        }
    }
}

impl LightningWallet {
    pub async fn open(account: AccountKey, storage_dir: &Path, network: LightningNetwork) -> Result<Self, LightningError> {
        fs::create_dir_all(storage_dir).map_err(|error| LightningError::Storage(error.to_string()))?;
        let path = storage_dir.join(DATABASE_FILE);
        let tree_store = SqliteTreeStore::new(&path.to_string_lossy())?;
        let database = Arc::new(WalletDatabase::open(&path)?);
        let signer = Arc::new(LockableSigner::new(&account));
        drop(account);
        let (cancel, cancel_receiver) = watch::channel(());
        let wallet = WalletBuilder::new(SparkWalletConfig::default_config(network.into()), Arc::new(SparkSignerAdapter::new(signer.clone())))
            .with_tree_store(Arc::new(tree_store))
            .with_session_store(database.clone())
            .with_cancellation_token(cancel_receiver)
            .build()
            .await?;
        let wallet = Arc::new(wallet);
        wallet.start_background_processing().await;
        Ok(Self {
            wallet,
            database,
            network,
            lock: SessionLock::new(signer, cancel),
        })
    }

    pub fn verify(&self, digest: &[u8; 32], signature: &[u8]) -> bool {
        verify_signature(&self.wallet.get_identity_public_key(), digest, signature)
    }

    pub fn address(&self) -> String {
        SparkAddress::new(self.wallet.get_identity_public_key().serialize(), self.network).encode()
    }

    pub async fn balance(&self) -> Result<u64, LightningError> {
        Ok(self.wallet.get_balance().await?)
    }

    pub async fn create_invoice(&self, amount_sats: Option<u64>, expiry_secs: Option<u32>) -> Result<LightningInvoice, LightningError> {
        let payment = self.wallet.create_lightning_invoice(amount_sats.unwrap_or(AMOUNTLESS_INVOICE_SATS), None, None, expiry_secs, false).await?;
        decode_invoice(&payment.invoice)
    }

    pub async fn prepare_send(&self, invoice: LightningInvoice, amount_sats: Option<u64>) -> Result<LightningSendQuote, LightningError> {
        let amount = send_amount(&invoice, amount_sats, self.network, now())?;
        let fee_sats = self.wallet.fetch_lightning_send_fee_estimate(&invoice.invoice, requested_amount(&invoice, amount)).await?;
        Ok(LightningSendQuote { invoice, amount_sats: amount, fee_sats })
    }

    pub async fn send(&self, quote: &LightningSendQuote) -> Result<String, LightningError> {
        let _permit = self.lock.begin_send()?;
        if let Some(pending) = self.database.get_pending_sends()?.into_iter().find(|send| send.invoice == quote.invoice.invoice)
            && let Some(transfer_id) = self.finish_pending_send(&pending).await?
        {
            return Ok(transfer_id);
        }
        let amount = send_amount(&quote.invoice, Some(quote.amount_sats), self.network, now())?;
        let amount_to_send = requested_amount(&quote.invoice, amount);
        let transfer_id = TransferId::generate();
        self.database.add_pending_send(&PendingSend {
            transfer_id: transfer_id.to_string(),
            invoice: quote.invoice.invoice.clone(),
            amount_sats: amount_to_send,
            expires_at: quote.invoice.expires_at,
        })?;
        let result = Box::pin(self.wallet.pay_lightning_invoice(&quote.invoice.invoice, amount_to_send, Some(quote.fee_sats), false, Some(transfer_id.clone()))).await?;
        let _ = self.database.delete_pending_send(&transfer_id.to_string());
        Ok(result.transfer.id.to_string())
    }

    pub async fn payments(&self, limit: u64, offset: u64) -> Result<Vec<LightningPayment>, LightningError> {
        let request = ListTransfersRequest {
            paging: Some(PagingFilter { offset, limit, order: Order::Descending }),
            transfer_ids: vec![],
        };
        let transfers = self.wallet.list_transfers(request).await?;
        Ok(transfers.items.iter().filter_map(map_transfer).collect())
    }

    pub async fn payment(&self, id: &str) -> Result<Option<LightningPayment>, LightningError> {
        let transfer_id = TransferId::from_str(id).map_err(|_| LightningError::InvalidTransfer)?;
        let request = ListTransfersRequest {
            paging: None,
            transfer_ids: vec![transfer_id.clone()],
        };
        let transfers = self.wallet.list_transfers(request).await?;
        Ok(transfers.items.iter().filter(|transfer| transfer.id == transfer_id).find_map(map_transfer))
    }

    pub async fn sync(&self) -> Result<(), LightningError> {
        let synced = self.wallet.sync().await.map_err(LightningError::from);
        let claimed = self.wallet.claim_pending_transfers().await.map(|_| ()).map_err(LightningError::from);
        let resumed = self.resume_pending_sends().await;
        synced.and(claimed).and(resumed)
    }

    pub fn lock(&self) {
        self.lock.request();
    }

    async fn finish_pending_send(&self, send: &PendingSend) -> Result<Option<String>, LightningError> {
        let transfer_id = TransferId::from_str(&send.transfer_id).map_err(LightningError::Storage)?;
        let resumed = Box::pin(self.wallet.resume_lightning_send(&transfer_id, &send.invoice, send.amount_sats)).await?;
        let exists = resumed.is_some() || self.payment(&send.transfer_id).await?.is_some();
        self.database.delete_pending_send(&send.transfer_id)?;
        Ok(exists.then(|| send.transfer_id.clone()))
    }

    async fn resume_pending_sends(&self) -> Result<(), LightningError> {
        for send in self.database.get_pending_sends()? {
            let _ = self.resume_pending_send(&send).await;
        }
        Ok(())
    }

    async fn resume_pending_send(&self, send: &PendingSend) -> Result<(), LightningError> {
        let transfer_id = TransferId::from_str(&send.transfer_id).map_err(LightningError::Storage)?;
        match Box::pin(self.wallet.resume_lightning_send(&transfer_id, &send.invoice, send.amount_sats)).await {
            Ok(_) => self.database.delete_pending_send(&send.transfer_id),
            Err(_) if send.expires_at <= now() => self.database.delete_pending_send(&send.transfer_id),
            Err(error) => Err(error.into()),
        }
    }
}

impl Drop for LightningWallet {
    fn drop(&mut self) {
        self.lock.apply();
    }
}

fn verify_signature(public_key: &PublicKey, digest: &[u8; 32], signature: &[u8]) -> bool {
    if signature.len() != SIGNATURE_LEN {
        return false;
    }
    let Ok(signature) = Signature::from_compact(&signature[..COMPACT_SIGNATURE_LEN]) else {
        return false;
    };
    Secp256k1::verification_only().verify_ecdsa(&Message::from_digest(*digest), &signature, public_key).is_ok()
}

impl From<LightningNetwork> for Network {
    fn from(network: LightningNetwork) -> Self {
        match network {
            LightningNetwork::Mainnet => Network::Mainnet,
            LightningNetwork::Regtest => Network::Regtest,
        }
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or_default()
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::hex::decode_hex;

    use super::*;
    use crate::keys::account_key;
    use crate::testkit::ABANDON_SEED_HEX;

    const TEST_IDENTITY_PUBLIC_KEY: &str = "02698b27ac308b275671b3ca25436346469d04a5bba578ae39feba1d65897a6abc";

    #[tokio::test]
    async fn test_lightning_wallet_regtest() {
        let dir = primitives::testkit::temp_dir::TempDir::new().unwrap();
        let seed = decode_hex(ABANDON_SEED_HEX).unwrap();
        let wallet = LightningWallet::open(account_key(&seed, LightningNetwork::Regtest).unwrap(), dir.path(), LightningNetwork::Regtest).await.unwrap();

        assert_eq!(wallet.wallet.get_identity_public_key().to_string(), TEST_IDENTITY_PUBLIC_KEY);
        wallet.sync().await.unwrap();
        wallet.balance().await.unwrap();
        wallet.payments(10, 0).await.unwrap();
        assert_eq!(wallet.payment(&TransferId::generate().to_string()).await.unwrap(), None);
        assert_eq!(wallet.payment("transfer").await, Err(LightningError::InvalidTransfer));

        let invoice = wallet.create_invoice(Some(1000), Some(600)).await.unwrap();
        assert_eq!((invoice.network, invoice.amount_sats), (LightningNetwork::Regtest, Some(1000)));
        assert_eq!(invoice.expires_at - invoice.created_at, 600);

        let amountless = wallet.create_invoice(None, None).await.unwrap();
        assert_eq!(amountless.amount_sats, None);
        wallet.lock();
        assert!(wallet.create_invoice(Some(1000), None).await.is_err());
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::secp256k1::SecretKey;
    use signer::{SignatureScheme, Signer};

    use super::*;
    use crate::keys::account_key;

    #[test]
    fn test_session_lock_waits_for_a_running_send() {
        let signer = Arc::new(LockableSigner::new(&account_key(&[7u8; 64], LightningNetwork::Regtest).unwrap()));
        let (cancel, _receiver) = watch::channel(());
        let lock = SessionLock::new(signer.clone(), cancel);

        let permit = lock.begin_send().unwrap();
        lock.request();

        assert!(!signer.is_locked());
        assert!(matches!(lock.begin_send(), Err(LightningError::Locked)));
        assert!(!signer.is_locked());

        drop(permit);

        assert!(signer.is_locked());
    }

    #[test]
    fn test_session_lock_locks_at_once_without_a_send() {
        let signer = Arc::new(LockableSigner::new(&account_key(&[7u8; 64], LightningNetwork::Regtest).unwrap()));
        let (cancel, _receiver) = watch::channel(());
        let lock = SessionLock::new(signer.clone(), cancel);

        drop(lock.begin_send().unwrap());
        assert!(!signer.is_locked());

        lock.request();

        assert!(signer.is_locked());
    }

    #[test]
    fn test_verify_signature() {
        let private_key = [7u8; 32];
        let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &SecretKey::from_slice(&private_key).unwrap());
        let other_key = PublicKey::from_secret_key(&Secp256k1::new(), &SecretKey::from_slice(&[8u8; 32]).unwrap());
        let digest = [1u8; 32];
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &digest, &private_key).unwrap();

        assert!(verify_signature(&public_key, &digest, &signature));
        assert!(!verify_signature(&other_key, &digest, &signature));
        assert!(!verify_signature(&public_key, &[2u8; 32], &signature));
        assert!(!verify_signature(&public_key, &digest, &signature[..10]));
        assert!(!verify_signature(&public_key, &digest, &signature[..COMPACT_SIGNATURE_LEN]));
        assert!(!verify_signature(&public_key, &digest, &[signature.as_slice(), &[0]].concat()));
    }
}
