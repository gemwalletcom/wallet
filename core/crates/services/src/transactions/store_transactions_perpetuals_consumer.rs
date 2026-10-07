use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use number_formatter::BigNumberFormatter;
use primitives::{Transaction, TransactionId, TransactionState};
use storage::TransactionPerpetualRecord;
use streamer::consumer::MessageConsumer;

use super::repository::Repository;

pub struct StoreTransactionsPerpetualsConsumer {
    repository: Arc<dyn Repository>,
}

impl StoreTransactionsPerpetualsConsumer {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    async fn perpetual_record(&self, transaction: &Transaction) -> Result<Option<TransactionPerpetualRecord>, Box<dyn Error + Send + Sync>> {
        if transaction.state != TransactionState::Confirmed {
            return Ok(None);
        }
        let Some(metadata) = transaction.perpetual_metadata() else {
            return Ok(None);
        };
        let (Some(referral_fee), Some(provider)) = (metadata.referral_fee, metadata.provider) else {
            return Ok(None);
        };
        let decimals = self.repository.asset(referral_fee.asset_id).await?.decimals;
        Ok(Some(TransactionPerpetualRecord {
            provider,
            asset_id: transaction.asset_id.clone(),
            kind: transaction.transaction_type.clone(),
            direction: metadata.direction,
            size_usd: BigNumberFormatter::value_as_f64(&transaction.value, decimals)?,
            pnl_usd: metadata.pnl,
            referral_fee_amount_usd: BigNumberFormatter::value_as_f64(&referral_fee.value, decimals)?,
            created_at: transaction.created_at.naive_utc(),
        }))
    }
}

#[async_trait]
impl MessageConsumer<TransactionId, usize> for StoreTransactionsPerpetualsConsumer {
    async fn should_consume(&self, _payload: &TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transaction = self.repository.transaction(payload.clone()).await?;
        let Some(record) = self.perpetual_record(&transaction).await? else {
            return Ok(0);
        };
        Ok(self.repository.upsert_transaction_perpetual(payload, record).await?)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{Asset, PerpetualProvider, TransactionPerpetualMetadata, TransactionSwapReferralFee, TransactionType};

    use super::*;
    use crate::testkit::MemoryTransactionsRepository;

    fn usdc() -> Asset {
        Asset { decimals: 6, ..Asset::mock_erc20() }
    }

    fn perpetual_transaction(state: TransactionState) -> Transaction {
        let metadata = TransactionPerpetualMetadata {
            pnl: 12.5,
            provider: Some(PerpetualProvider::Hypercore),
            referral_fee: Some(TransactionSwapReferralFee {
                asset_id: usdc().id,
                value: BigUint::from(2_000_000u64),
            }),
            ..TransactionPerpetualMetadata::mock()
        };
        Transaction {
            transaction_type: TransactionType::PerpetualOpenPosition,
            state,
            value: BigUint::from(150_000_000u64),
            metadata: serde_json::to_value(metadata).ok(),
            ..Transaction::mock()
        }
    }

    #[tokio::test]
    async fn test_confirmed_referral_is_recorded_in_quote_units() {
        let transaction = perpetual_transaction(TransactionState::Confirmed);
        let repository = Arc::new(MemoryTransactionsRepository::new(vec![transaction.clone()], vec![usdc()]));
        let consumer = StoreTransactionsPerpetualsConsumer::new(repository.clone());

        assert_eq!(consumer.consume(transaction.id.clone()).await.unwrap(), 1);

        let records = repository.perpetual_records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, transaction.id);
        assert_eq!(records[0].1.size_usd, 150.0);
        assert_eq!(records[0].1.pnl_usd, 12.5);
        assert_eq!(records[0].1.referral_fee_amount_usd, 2.0);
        assert_eq!(records[0].1.asset_id, transaction.asset_id);
    }

    #[tokio::test]
    async fn test_unconfirmed_perpetual_is_skipped() {
        let transaction = perpetual_transaction(TransactionState::Pending);
        let repository = Arc::new(MemoryTransactionsRepository::new(vec![transaction.clone()], vec![usdc()]));
        let consumer = StoreTransactionsPerpetualsConsumer::new(repository.clone());

        assert_eq!(consumer.consume(transaction.id).await.unwrap(), 0);
        assert!(repository.perpetual_records().is_empty());
    }
}
