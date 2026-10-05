use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetId, DAY, PriceData, SwapProvider, Transaction, TransactionId, TransactionType, swap::SwapStatus};
use storage::TransactionSwapRecord;
use streamer::consumer::MessageConsumer;

use crate::ConfigCacher;
use crate::transactions::StoreTransactionsSwapsConsumerConfig;
use crate::transactions::repository::{AssetPriceHistory, Repository};

struct AssetValue {
    amount: f64,
    amount_usd: Option<f64>,
    is_enabled: bool,
}

pub struct StoreTransactionsSwapsConsumer {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
}

impl StoreTransactionsSwapsConsumer {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>) -> Self {
        Self { repository, config }
    }

    async fn swap_record(&self, config: &StoreTransactionsSwapsConsumerConfig, transaction: &Transaction) -> Result<Option<TransactionSwapRecord>, Box<dyn Error + Send + Sync>> {
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
        let fee = self.asset_value(&referral_fee.asset_id, &referral_fee.value, at).await?;
        if !fee.is_enabled {
            return Ok(None);
        }
        let from = self.asset_value(&metadata.from_asset, &metadata.from_value, at).await?;
        let to = self.asset_value(&metadata.to_asset, &metadata.to_value, at).await?;
        let to_amount_usd = to.amount_usd.filter(|value| is_output_within_input_value(*value, from.amount_usd, config.max_output_to_input_value));
        let from_amount_usd = from.amount_usd.filter(|value| is_input_within_output_value(*value, to_amount_usd, config.max_input_to_output_value));
        let referral_fee_amount_usd = fee.amount_usd.filter(|value| is_fee_within_swap_value(*value, from_amount_usd.or(to_amount_usd)));
        Ok(Some(TransactionSwapRecord {
            provider,
            status,
            from_asset_id: metadata.from_asset,
            from_amount: from.amount,
            from_amount_usd,
            to_asset_id: metadata.to_asset,
            to_amount: to.amount,
            to_amount_usd,
            referral_fee_asset_id: referral_fee.asset_id,
            referral_fee_amount_usd,
            created_at: at,
        }))
    }

    async fn asset_value(&self, asset_id: &AssetId, value: &BigUint, at: NaiveDateTime) -> Result<AssetValue, Box<dyn Error + Send + Sync>> {
        let AssetPriceHistory { assets, price_at: price, prices } = self.repository.asset_price_history(asset_id.clone(), at).await?;
        let asset = assets.into_iter().next().ok_or_else(|| format!("asset {asset_id} not found"))?;
        let amount = BigNumberFormatter::value_as_f64(value, asset.asset.decimals as u32)?;
        let max_age = Duration::from_std(DAY)?;
        let is_enabled = asset.properties.is_enabled;
        let amount_usd = price.filter(|(price_at, _)| is_enabled && at - *price_at <= max_age && is_within_supply(amount, &prices)).map(|(_, price)| amount * price);
        Ok(AssetValue { amount, amount_usd, is_enabled })
    }
}

#[async_trait]
impl MessageConsumer<TransactionId, usize> for StoreTransactionsSwapsConsumer {
    async fn should_consume(&self, _payload: &TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transaction = self.repository.transaction(payload.clone()).await?;
        let config = StoreTransactionsSwapsConsumerConfig::read(&self.config).await?;
        let Some(record) = self.swap_record(&config, &transaction).await? else {
            return Ok(0);
        };
        Ok(self.repository.upsert_transaction_swap(payload, record).await?)
    }
}

fn is_within_supply(amount: f64, prices: &[PriceData]) -> bool {
    prices.iter().flat_map(|price| [price.total_supply, price.max_supply]).flatten().reduce(f64::max).is_none_or(|supply| amount <= supply)
}

fn is_output_within_input_value(output_usd: f64, input_usd: Option<f64>, max_output_to_input_value: f64) -> bool {
    input_usd.is_none_or(|input_usd| output_usd <= input_usd * max_output_to_input_value)
}

fn is_input_within_output_value(input_usd: f64, output_usd: Option<f64>, max_input_to_output_value: f64) -> bool {
    output_usd.is_none_or(|output_usd| input_usd <= output_usd * max_input_to_output_value)
}

fn is_fee_within_swap_value(fee_usd: f64, swap_usd: Option<f64>) -> bool {
    swap_usd.is_none_or(|swap_usd| fee_usd <= swap_usd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_output_within_input_value() {
        assert!(is_output_within_input_value(99.0, Some(100.0), 2.0));
        assert!(!is_output_within_input_value(2.46e18, Some(12.3), 2.0));
        assert!(is_output_within_input_value(2.46e18, None, 2.0));
    }

    #[test]
    fn test_is_input_within_output_value() {
        assert!(is_input_within_output_value(100.0, Some(99.0), 2.0));
        assert!(!is_input_within_output_value(561510226.8, Some(9.15), 2.0));
        assert!(is_input_within_output_value(561510226.8, None, 2.0));
    }

    #[test]
    fn test_is_fee_within_swap_value() {
        assert!(is_fee_within_swap_value(1.5, Some(100.0)));
        assert!(!is_fee_within_swap_value(81.86, Some(0.12)));
        assert!(is_fee_within_swap_value(5.0, None));
    }

    #[test]
    fn test_is_within_supply() {
        let price = PriceData {
            total_supply: Some(1_000_000.0),
            max_supply: None,
            ..PriceData::mock()
        };
        assert!(is_within_supply(5_000.0, std::slice::from_ref(&price)));
        assert!(!is_within_supply(1e20, std::slice::from_ref(&price)));
        assert!(is_within_supply(
            1e20,
            &[PriceData {
                total_supply: None,
                max_supply: None,
                ..PriceData::mock()
            }]
        ));
        assert!(is_within_supply(1e20, &[]));
    }
}
