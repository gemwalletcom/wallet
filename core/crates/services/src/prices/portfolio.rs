use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use number_formatter::BigNumberFormatter;
use primitives::{ChartPeriod, ChartValue, ChartValuePercentage, PortfolioAllocation, PortfolioAsset, PortfolioAssets};

use super::repository::{PortfolioPrice, Repository};

use crate::ConfigCacher;

pub struct PortfolioClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
}

struct ResolvedAsset {
    asset: PortfolioAsset,
    balance: f64,
    charts: Vec<storage::ChartResult>,
    current_value: f64,
}

impl PortfolioClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>) -> Self {
        Self { repository, config }
    }

    pub async fn get_portfolio_charts(&self, assets: Vec<PortfolioAsset>, period: ChartPeriod) -> Result<PortfolioAssets, Box<dyn Error + Send + Sync>> {
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let asset_ids = assets.iter().map(|asset| asset.asset_id.clone()).collect();
        let prices = self.repository.portfolio_prices(asset_ids, period, primary_price_max_age).await?;
        let assets: Vec<ResolvedAsset> = assets.into_iter().zip(prices).filter_map(|(input, price)| Self::resolved_asset(input, price?)).collect();
        let chart_data = Self::chart_values(&assets);
        Ok(Self::build_portfolio(assets, chart_data))
    }

    fn chart_values(assets: &[ResolvedAsset]) -> BTreeMap<i64, f64> {
        assets
            .iter()
            .flat_map(|r| r.charts.iter().map(|(ts, price)| (ts.and_utc().timestamp(), r.balance * price)))
            .fold(BTreeMap::new(), |mut acc, (ts, value)| {
                *acc.entry(ts).or_default() += value;
                acc
            })
    }

    fn build_portfolio(assets: Vec<ResolvedAsset>, chart_data: BTreeMap<i64, f64>) -> PortfolioAssets {
        let values: Vec<ChartValue> = chart_data.into_iter().map(|(ts, value)| ChartValue { timestamp: ts as i32, value: value as f32 }).collect();

        let cmp = |a: &&ChartValue, b: &&ChartValue| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal);
        let all_time_high = values.iter().max_by(cmp).cloned();
        let all_time_low = values.iter().min_by(cmp).cloned();

        let total_value: f64 = assets.iter().map(|r| r.current_value).sum();
        let total_value_f32 = total_value as f32;

        let to_percentage = |cv: &ChartValue| ChartValuePercentage {
            date: chrono::DateTime::from_timestamp(cv.timestamp as i64, 0).unwrap_or_default(),
            value: cv.value,
            percentage: if total_value_f32 > 0.0 { (cv.value - total_value_f32) / total_value_f32 * 100.0 } else { 0.0 },
        };

        let allocation: Vec<PortfolioAllocation> = assets
            .into_iter()
            .map(|r| PortfolioAllocation {
                asset_id: r.asset.asset_id,
                value: r.current_value as f32,
                percentage: if total_value > 0.0 { (r.current_value / total_value) as f32 } else { 0.0 },
            })
            .collect();

        PortfolioAssets {
            total_value: total_value_f32,
            values,
            all_time_high: all_time_high.as_ref().map(to_percentage),
            all_time_low: all_time_low.as_ref().map(to_percentage),
            allocation,
        }
    }

    fn resolved_asset(input: PortfolioAsset, price: PortfolioPrice) -> Option<ResolvedAsset> {
        let balance = BigNumberFormatter::value_as_f64(&input.value, price.asset.decimals).ok()?;
        Some(ResolvedAsset {
            asset: input,
            balance,
            charts: price.charts,
            current_value: balance * price.price,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{Asset, AssetId, Chain};

    use super::*;
    use crate::testkit::{MemoryConfigRepository, MemoryPricesRepository};

    #[tokio::test]
    async fn test_portfolio_skips_unpriced_assets() {
        let at = DateTime::from_timestamp(1_700_000_000, 0).unwrap().naive_utc();
        let bitcoin = PortfolioPrice {
            asset: Asset::mock_btc(),
            price: 100.0,
            charts: vec![(at, 90.0)],
        };
        let repository = MemoryPricesRepository::default().with_portfolio(vec![Some(bitcoin), None]);
        let client = PortfolioClient::new(Arc::new(repository), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));
        let assets = vec![
            PortfolioAsset {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                value: BigUint::from(200_000_000u64),
            },
            PortfolioAsset {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                value: BigUint::from(1u64),
            },
        ];

        let portfolio = client.get_portfolio_charts(assets, ChartPeriod::Day).await.unwrap();

        assert_eq!(portfolio.total_value, 200.0);
        assert_eq!(portfolio.values, vec![ChartValue { timestamp: 1_700_000_000, value: 180.0 }]);
        assert_eq!(portfolio.allocation.len(), 1);
        assert_eq!(portfolio.allocation[0].asset_id, AssetId::from_chain(Chain::Bitcoin));
        assert_eq!(portfolio.allocation[0].percentage, 1.0);
    }
}
