use std::error::Error;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, DAY, SwapProvider, Transaction, TransactionId, TransactionType, swap::SwapStatus};
use storage::{AssetsRepository, Database, PricesRepository, TransactionSwapRecord, TransactionsRepository, TransactionsSwapsRepository};
use streamer::consumer::MessageConsumer;

pub struct StoreTransactionsSwapsConsumer {
    database: Database,
}

impl StoreTransactionsSwapsConsumer {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    async fn swap_record(&self, transaction: &Transaction) -> Result<Option<TransactionSwapRecord>, Box<dyn Error + Send + Sync>> {
        let status = transaction.state.swap_status();
        if transaction.transaction_type != TransactionType::Swap || status == SwapStatus::Pending {
            return Ok(None);
        }
        let Some(metadata) = transaction.swap_metadata() else {
            return Ok(None);
        };
        let (Some(referral_fee), Some(provider)) = (metadata.referral_fee, metadata.provider.and_then(|provider| provider.parse::<SwapProvider>().ok())) else {
            return Ok(None);
        };
        let at = transaction.created_at.naive_utc();
        Ok(Some(TransactionSwapRecord {
            provider,
            status,
            from_amount_usd: self.amount_usd(&metadata.from_asset, &metadata.from_value, at).await?,
            from_asset_id: metadata.from_asset,
            to_amount_usd: self.amount_usd(&metadata.to_asset, &metadata.to_value, at).await?,
            to_asset_id: metadata.to_asset,
            referral_fee_amount_usd: self.amount_usd(&referral_fee.asset_id, &referral_fee.value, at).await?,
            referral_fee_asset_id: referral_fee.asset_id,
        }))
    }

    async fn amount_usd(&self, asset_id: &AssetId, value: &BigUint, at: NaiveDateTime) -> Result<Option<f64>, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.clone();
        let (assets, price) = self
            .database
            .run(move |client| Ok::<_, storage::DatabaseError>((client.get_assets(vec![asset_id.clone()])?, client.get_price_at(&asset_id, at)?)))
            .await?;
        let (Some(asset), Some((price_at, price))) = (assets.first(), price) else {
            return Ok(None);
        };
        if at - price_at > chrono::Duration::from_std(DAY)? {
            return Ok(None);
        }
        Ok(Some(BigNumberFormatter::value_as_f64(&value.to_string(), asset.decimals as u32)? * price))
    }
}

#[async_trait]
impl MessageConsumer<TransactionId, usize> for StoreTransactionsSwapsConsumer {
    async fn should_consume(&self, _payload: &TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let id = payload.clone();
        let transaction = self.database.run(move |client| client.get_transaction_by_id(&id, vec![])).await?;
        let Some(record) = self.swap_record(&transaction).await? else {
            return Ok(0);
        };
        Ok(self.database.run(move |client| client.upsert_transaction_swap(&payload, record)).await?)
    }
}
