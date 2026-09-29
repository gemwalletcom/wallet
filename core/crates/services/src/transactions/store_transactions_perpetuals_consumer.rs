use std::error::Error;

use async_trait::async_trait;
use number_formatter::BigNumberFormatter;
use primitives::{Transaction, TransactionId, TransactionState};
use storage::{AssetsRepository, Database, TransactionPerpetualRecord, TransactionsPerpetualsRepository, TransactionsRepository};
use streamer::consumer::MessageConsumer;

pub struct StoreTransactionsPerpetualsConsumer {
    database: Database,
}

impl StoreTransactionsPerpetualsConsumer {
    pub fn new(database: Database) -> Self {
        Self { database }
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
        let quote_asset_id = referral_fee.asset_id;
        let decimals = self.database.run(move |client| client.get_asset(&quote_asset_id)).await?.decimals as u32;
        Ok(Some(TransactionPerpetualRecord {
            provider,
            asset_id: transaction.asset_id.clone(),
            direction: metadata.direction,
            size_usd: BigNumberFormatter::value_as_f64(&transaction.value.to_string(), decimals)?,
            referral_fee_amount_usd: BigNumberFormatter::value_as_f64(&referral_fee.value.to_string(), decimals)?,
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
        let id = payload.clone();
        let transaction = self.database.run(move |client| client.get_transaction_by_id(&id, vec![])).await?;
        let Some(record) = self.perpetual_record(&transaction).await? else {
            return Ok(0);
        };
        Ok(self.database.run(move |client| client.upsert_transaction_perpetual(&payload, record)).await?)
    }
}
