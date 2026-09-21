use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use num_bigint::BigInt;
use primitives::{RecentActivityType, TransactionInputType, TransactionType, WalletId};

use super::{GemPendingTransactionInput, GemRecentActivity, GemRecentActivityService, GemRecentActivityStore, GemRecipient, GemTransferData};
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use crate::services::error::GemServiceError;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};

impl GemTransferData {
    pub fn mock(input_type: TransactionInputType) -> Self {
        Self {
            input_type,
            recipient: GemRecipient {
                address: "recipient".to_string(),
                name: None,
                memo: Some("memo".to_string()),
                references: vec![],
            },
            value: BigInt::from(10),
            use_max_amount: false,
        }
    }
}

impl GemPendingTransactionInput {
    pub fn mock(input_type: TransactionInputType, transaction_type: TransactionType, hash: &str, transaction_index: u32, transaction_count: u32) -> Self {
        Self {
            sender: "sender".to_string(),
            transfer: GemTransferData::mock(input_type),
            value: BigInt::from(99),
            transaction_type,
            hash: hash.to_string(),
            fee: GemTransactionLoadFee::mock(1),
            network_fee: BigInt::from(1),
            metadata: GemTransactionLoadMetadata::None,
            simulation: None,
            transaction_index,
            transaction_count,
        }
    }
}

impl GemRecentActivityService {
    pub fn mock(store: Arc<dyn GemRecentActivityStore>, wallet_id: Option<WalletId>) -> Self {
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), Arc::new(MemoryWalletStore::default())));
        session.set_current_wallet_id(wallet_id).unwrap();
        Self::new(store, session)
    }
}

#[derive(Default)]
pub struct MemoryRecentActivityStore {
    pub added: Mutex<Vec<(GemRecentActivity, WalletId)>>,
}

#[async_trait]
impl GemRecentActivityStore for MemoryRecentActivityStore {
    async fn add(&self, activity: GemRecentActivity, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.added.lock().unwrap().push((activity, wallet_id));
        Ok(())
    }
    async fn clear(&self, _wallet_id: WalletId, _types: Vec<RecentActivityType>) -> Result<(), GemServiceError> {
        self.added.lock().unwrap().clear();
        Ok(())
    }
}
