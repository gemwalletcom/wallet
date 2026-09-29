use std::error::Error;

use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, DAY, SwapProvider, Transaction, TransactionId, TransactionType, swap::SwapStatus};
use storage::{AssetsRepository, Database, DatabaseError, PricesRepository, TransactionSwapRecord, TransactionsRepository, TransactionsSwapsRepository};
use streamer::consumer::MessageConsumer;

const MAX_OUTPUT_TO_INPUT_VALUE: f64 = 2.0;

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
        let (from_amount, from_amount_usd) = self.amount(&metadata.from_asset, &metadata.from_value, at).await?;
        let (to_amount, to_amount_usd) = self.amount(&metadata.to_asset, &metadata.to_value, at).await?;
        let (_, referral_fee_amount_usd) = self.amount(&referral_fee.asset_id, &referral_fee.value, at).await?;
        let to_amount_usd = value_within(to_amount_usd, from_amount_usd, MAX_OUTPUT_TO_INPUT_VALUE);
        let referral_fee_amount_usd = value_within(referral_fee_amount_usd, from_amount_usd.or(to_amount_usd), 1.0);
        Ok(Some(TransactionSwapRecord {
            provider,
            status,
            from_asset_id: metadata.from_asset,
            from_amount,
            from_amount_usd,
            to_asset_id: metadata.to_asset,
            to_amount,
            to_amount_usd,
            referral_fee_asset_id: referral_fee.asset_id,
            referral_fee_amount_usd,
            created_at: at,
        }))
    }

    async fn amount(&self, asset_id: &AssetId, value: &BigUint, at: NaiveDateTime) -> Result<(f64, Option<f64>), Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.clone();
        let (asset, price) = self.database.run(move |client| Ok::<_, DatabaseError>((client.get_asset(&asset_id)?, client.get_price_at(&asset_id, at)?))).await?;
        let amount = BigNumberFormatter::value_as_f64(&value.to_string(), asset.decimals as u32)?;
        let max_age = Duration::from_std(DAY)?;
        let amount_usd = price.filter(|(price_at, _)| at - *price_at <= max_age).map(|(_, price)| amount * price);
        Ok((amount, amount_usd))
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

fn value_within(value: Option<f64>, reference: Option<f64>, max_ratio: f64) -> Option<f64> {
    value.filter(|value| reference.is_none_or(|reference| *value <= reference * max_ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_within() {
        assert_eq!(value_within(Some(1.5), Some(100.0), 1.0), Some(1.5));
        assert_eq!(value_within(Some(1.24e16), Some(12.3), 1.0), None);
        assert_eq!(value_within(Some(2.46e18), Some(12.3), MAX_OUTPUT_TO_INPUT_VALUE), None);
        assert_eq!(value_within(Some(99.0), Some(100.0), MAX_OUTPUT_TO_INPUT_VALUE), Some(99.0));
        assert_eq!(value_within(Some(5.0), None, 1.0), Some(5.0));
        assert_eq!(value_within(None, Some(100.0), 1.0), None);
    }
}
