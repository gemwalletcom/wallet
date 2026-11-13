use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::secp256k1::{Message, PublicKey, Secp256k1};
use primitives::time::unix_timestamp;
use spark_sqlite::SqliteTreeStore;
use spark_wallet::{ListTransfersRequest, Network, Order, PagingFilter, SparkSignerAdapter, SparkWallet, SparkWalletConfig, TransferId, WalletBuilder};
use tokio::sync::watch;

use crate::address::SparkAddress;
use crate::error::LightningError;
use crate::invoice::{requested_amount, send_amount};
use crate::keys::AccountKey;
use crate::mapper::map_transfer;
use crate::model::{LightningInvoice, LightningPayment, LightningPaymentStatus, LightningSendQuote};
use crate::signer::LockableSigner;
use crate::store::{PendingSend, WalletDatabase};

const DATABASE_FILE: &str = "spark.sqlite";
const AMOUNTLESS_INVOICE_SATS: u64 = 0;
const SIGNATURE_LEN: usize = 65;
const COMPACT_SIGNATURE_LEN: usize = 64;

static IN_FLIGHT: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Mutex::default);

pub struct LightningWallet {
    wallet: Arc<SparkWallet>,
    database: WalletDatabase,
    lock: SessionLock,
}

enum PendingSendState {
    Sent,
    Unsent,
}

struct InFlight(String);

impl InFlight {
    fn begin(transfer_id: &str) -> Self {
        IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner).insert(transfer_id.to_string());
        Self(transfer_id.to_string())
    }

    fn contains(transfer_id: &str) -> bool {
        IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner).contains(transfer_id)
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner).remove(&self.0);
    }
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
    pub async fn open(account: AccountKey, storage_dir: &Path) -> Result<Self, LightningError> {
        fs::create_dir_all(storage_dir).map_err(|error| LightningError::Storage(error.to_string()))?;
        let path = storage_dir.join(DATABASE_FILE);
        let tree_store = SqliteTreeStore::new(&path.to_string_lossy())?;
        let database = WalletDatabase::open(&path)?;
        let signer = Arc::new(LockableSigner::new(&account));
        drop(account);
        let (cancel, cancel_receiver) = watch::channel(());
        let wallet = WalletBuilder::new(SparkWalletConfig::default_config(Network::Mainnet), Arc::new(SparkSignerAdapter::new(signer.clone())))
            .with_tree_store(Arc::new(tree_store))
            .with_cancellation_token(cancel_receiver)
            .build()
            .await?;
        let wallet = Arc::new(wallet);
        wallet.start_background_processing().await;
        Ok(Self {
            wallet,
            database,
            lock: SessionLock::new(signer, cancel),
        })
    }

    pub fn verify(&self, digest: &[u8; 32], signature: &[u8]) -> bool {
        verify_signature(&self.wallet.get_identity_public_key(), digest, signature)
    }

    pub fn address(&self) -> String {
        SparkAddress::new(self.wallet.get_identity_public_key().serialize()).encode()
    }

    pub async fn balance(&self) -> Result<u64, LightningError> {
        Ok(self.wallet.get_balance().await?)
    }

    pub async fn create_invoice(&self, amount_sats: Option<u64>) -> Result<String, LightningError> {
        Ok(self.wallet.create_lightning_invoice(amount_sats.unwrap_or(AMOUNTLESS_INVOICE_SATS), None, None, None, false).await?.invoice)
    }

    pub async fn send_fee(&self, invoice: &LightningInvoice, amount_sats: u64) -> Result<u64, LightningError> {
        let amount = send_amount(invoice, amount_sats, unix_timestamp())?;
        Ok(self.wallet.fetch_lightning_send_fee_estimate(&invoice.invoice, requested_amount(invoice, amount)).await?)
    }

    pub async fn send(&self, quote: &LightningSendQuote) -> Result<String, LightningError> {
        let _permit = self.lock.begin_send()?;
        let amount_to_send = requested_amount(&quote.invoice, send_amount(&quote.invoice, quote.amount_sats, unix_timestamp())?);
        if let Some(pending) = self.database.get_pending_sends()?.into_iter().find(|send| send.invoice == quote.invoice.invoice) {
            if let (Some(recorded), Some(requested)) = (pending.amount_sats, amount_to_send)
                && recorded != requested
            {
                return Err(LightningError::AmountMismatch { invoice: recorded, requested });
            }
            match self.resume(&pending).await {
                Ok(PendingSendState::Unsent) => {}
                Ok(PendingSendState::Sent) | Err(_) => return Ok(pending.transfer_id),
            }
        }
        let transfer_id = TransferId::generate();
        let id = transfer_id.to_string();
        let _in_flight = InFlight::begin(&id);
        self.database.add_pending_send(&PendingSend {
            transfer_id: id.clone(),
            invoice: quote.invoice.invoice.clone(),
            amount_sats: amount_to_send,
            expires_at: quote.invoice.expires_at,
        })?;
        match Box::pin(self.wallet.pay_lightning_invoice(&quote.invoice.invoice, amount_to_send, Some(quote.fee_sats), false, Some(transfer_id))).await {
            Ok(result) => Ok(result.transfer.id.to_string()),
            Err(error) => match self.payment(&id).await {
                Ok(None) => {
                    let _ = self.database.delete_pending_send(&id);
                    Err(error.into())
                }
                Ok(Some(_)) | Err(_) => Ok(id),
            },
        }
    }

    pub async fn payments(&self, limit: u64) -> Result<Vec<LightningPayment>, LightningError> {
        let request = ListTransfersRequest {
            paging: Some(PagingFilter { offset: 0, limit, order: Order::Descending }),
            transfer_ids: vec![],
        };
        let transfers = self.wallet.list_transfers(request).await?;
        Ok(transfers.items.iter().filter_map(map_transfer).collect())
    }

    pub async fn payment_status(&self, id: &str) -> Result<LightningPaymentStatus, LightningError> {
        if let Some(payment) = self.payment(id).await? {
            return Ok(payment.status);
        }
        let is_recorded = self.database.get_pending_sends()?.iter().any(|send| send.transfer_id == id);
        Ok(if is_recorded { LightningPaymentStatus::Pending } else { LightningPaymentStatus::Failed })
    }

    async fn payment(&self, id: &str) -> Result<Option<LightningPayment>, LightningError> {
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

    async fn resume_pending_sends(&self) -> Result<(), LightningError> {
        for send in self.database.get_pending_sends()? {
            let _ = self.resume(&send).await;
        }
        Ok(())
    }

    async fn resume(&self, send: &PendingSend) -> Result<PendingSendState, LightningError> {
        if InFlight::contains(&send.transfer_id) {
            return Ok(PendingSendState::Sent);
        }
        if send.expires_at <= unix_timestamp() {
            self.database.delete_pending_send(&send.transfer_id)?;
            return Ok(PendingSendState::Unsent);
        }
        let transfer_id = TransferId::from_str(&send.transfer_id).map_err(LightningError::Storage)?;
        let state = match Box::pin(self.wallet.resume_lightning_send(&transfer_id, &send.invoice, send.amount_sats)).await? {
            Some(_) => PendingSendState::Sent,
            None => match self.payment(&send.transfer_id).await?.map(|payment| payment.status) {
                Some(LightningPaymentStatus::Pending | LightningPaymentStatus::Completed) => PendingSendState::Sent,
                Some(LightningPaymentStatus::Failed) | None => PendingSendState::Unsent,
            },
        };
        match state {
            PendingSendState::Sent => {}
            PendingSendState::Unsent => self.database.delete_pending_send(&send.transfer_id)?,
        }
        Ok(state)
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

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::hex::decode_hex;

    use super::*;
    use crate::invoice::decode_invoice;
    use crate::keys::account_key;
    use crate::testkit::{ABANDON_SEED_HEX, IDENTITY_PUBLIC_KEY};

    #[tokio::test]
    async fn test_lightning_wallet() {
        let dir = primitives::testkit::temp_dir::TempDir::new().unwrap();
        let wallet = LightningWallet::open(account_key(&decode_hex(ABANDON_SEED_HEX).unwrap()).unwrap(), dir.path()).await.unwrap();

        assert_eq!(wallet.wallet.get_identity_public_key().to_string(), IDENTITY_PUBLIC_KEY);
        wallet.sync().await.unwrap();
        wallet.balance().await.unwrap();
        wallet.payments(10).await.unwrap();
        assert_eq!(wallet.payment(&TransferId::generate().to_string()).await.unwrap(), None);
        assert_eq!(wallet.payment("transfer").await, Err(LightningError::InvalidTransfer));

        assert_eq!(decode_invoice(&wallet.create_invoice(Some(1000)).await.unwrap()).unwrap().amount_sats, Some(1000));
        assert_eq!(decode_invoice(&wallet.create_invoice(None).await.unwrap()).unwrap().amount_sats, None);
        wallet.lock();
        assert!(wallet.create_invoice(Some(1000)).await.is_err());
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
        let signer = Arc::new(LockableSigner::new(&account_key(&[7u8; 64]).unwrap()));
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
        let signer = Arc::new(LockableSigner::new(&account_key(&[7u8; 64]).unwrap()));
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
