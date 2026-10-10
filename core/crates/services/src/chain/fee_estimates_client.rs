use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;

use cacher::FeeEstimatesCacher;
use chain_providers::{TransactionFeeEstimate, TransactionFeeEstimates};
use config_keys::ConfigParamKey;
use number_formatter::{BigNumberFormatter, CryptoFiatConverter};
use primitives::{Asset, Chain, ChainFeeEstimates, FeeEstimate, FeeEstimatesByPriority, FeeUnitType};

use super::chain_client::ChainClient;
use crate::ConfigCacher;
use crate::assets::AssetsClient;
use crate::prices::PriceClient;

pub struct FeeEstimatesClient {
    chain_client: ChainClient,
    assets_client: AssetsClient,
    price_client: PriceClient,
    cacher: Arc<dyn FeeEstimatesCacher>,
    config: Arc<ConfigCacher>,
}

impl FeeEstimatesClient {
    pub fn new(chain_client: ChainClient, assets_client: AssetsClient, price_client: PriceClient, cacher: Arc<dyn FeeEstimatesCacher>, config: Arc<ConfigCacher>) -> Self {
        Self {
            chain_client,
            assets_client,
            price_client,
            cacher,
            config,
        }
    }

    pub async fn get_chain_fee_estimates(&self, chain: Chain) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
        let duration = self.config.get_param_duration(&ConfigParamKey::TransactionsFeeEstimatesCacheDuration(chain)).await?;
        self.cacher.get_or_fetch_estimates(chain, duration, Box::pin(self.fetch_estimates(chain))).await
    }

    async fn fetch_estimates(&self, chain: Chain) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
        let estimates = self.chain_client.get_transaction_fee_estimates(chain).await?;
        let asset = self.assets_client.get_asset(&estimates.fee_asset).await?;
        let price = self.price_client.get_cache_price(&estimates.fee_asset).await?;
        map_fee_estimates(asset, estimates, price.price.price)
    }

    pub async fn get_fee_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .cacher
            .all_estimates()
            .await?
            .into_iter()
            .enumerate()
            .map(|(position, estimates)| ((Reverse(estimates.asset.chain().rank()), position), estimates))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect())
    }
}

fn map_fee_estimates(asset: Asset, estimates: TransactionFeeEstimates, price_usd: f64) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
    let chain = asset.chain();
    let rate_unit = chain.fee_unit_type();
    let asset_decimals = asset.decimals;
    let rate_decimals = match rate_unit {
        FeeUnitType::Native => asset_decimals,
        FeeUnitType::SatVb | FeeUnitType::Gwei => rate_unit.decimals(),
    };
    let map_estimates = |estimates| map_estimates_by_priority(estimates, rate_decimals, asset_decimals, price_usd);
    Ok(ChainFeeEstimates {
        transfer: map_estimates(estimates.transfer)?,
        token_transfer: estimates.token_transfer.map(map_estimates).transpose()?,
        swap: estimates.swap.map(map_estimates).transpose()?,
        block_time: chain.block_time(),
        token_type: chain.default_asset_type(),
        asset,
        rate_unit,
    })
}

fn map_estimates_by_priority(estimates: Vec<TransactionFeeEstimate>, rate_decimals: u32, asset_decimals: u32, price_usd: f64) -> Result<FeeEstimatesByPriority, Box<dyn Error + Send + Sync>> {
    estimates
        .into_iter()
        .map(|estimate| {
            Ok((
                estimate.priority,
                FeeEstimate {
                    base: BigNumberFormatter::value(estimate.gas_price_type.gas_price(), rate_decimals)?,
                    priority_fee: BigNumberFormatter::value(estimate.gas_price_type.priority_fee(), rate_decimals)?,
                    value: BigNumberFormatter::value(&estimate.fee, asset_decimals)?,
                    fiat_value: CryptoFiatConverter::to_fiat(&estimate.fee, asset_decimals, price_usd)?,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc, time::Duration};

    use chain_providers::{ChainProviders, TransactionFeeEstimate, TransactionFeeEstimates};
    use chrono::DateTime;
    use config_keys::ConfigParamKey;
    use primitives::{Asset, AssetMarket, AssetPriceInfo, Chain, ChainFeeEstimates, FeePriority, GasPriceType, Price, PriceProvider};
    use settings::Settings;

    use super::{FeeEstimatesClient, map_fee_estimates};
    use crate::ConfigCacher;
    use crate::assets::AssetsClient;
    use crate::chain::ChainClient;
    use crate::prices::PriceClient;
    use crate::testkit::{MemoryAssetRepository, MemoryConfigRepository, MemoryFeeEstimatesCacher, MemoryPriceCacher, MemoryPricesRepository, UnusedObservedCacher};

    fn client(cacher: Arc<MemoryFeeEstimatesCacher>) -> FeeEstimatesClient {
        let settings = Settings::new_setting_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Settings.yaml")).unwrap();
        let config = Arc::new(ConfigCacher::new(Arc::new(
            MemoryConfigRepository::new().with_value(&ConfigParamKey::TransactionsFeeEstimatesCacheDuration(Chain::Tron).key(), "2m"),
        )));
        let asset = Asset::from_chain(Chain::Tron);
        let price = AssetPriceInfo {
            asset_id: asset.id.clone(),
            price: Price::new(0.3, 0.0, DateTime::from_timestamp(0, 0).unwrap(), PriceProvider::Coingecko),
            market: AssetMarket::default(),
        };
        FeeEstimatesClient::new(
            ChainClient::new(ChainProviders::for_chain(Chain::Tron, &settings, "test")),
            AssetsClient::new(Arc::new(MemoryAssetRepository::new(vec![asset.as_basic_primitive()])), config.clone()),
            PriceClient::new(Arc::new(MemoryPricesRepository::default()), config.clone(), Arc::new(MemoryPriceCacher::new(vec![price])), Arc::new(UnusedObservedCacher)),
            cacher,
            config,
        )
    }

    fn cached(chain: Chain) -> ChainFeeEstimates {
        ChainFeeEstimates {
            asset: Asset::from_chain(chain),
            rate_unit: chain.fee_unit_type(),
            block_time: chain.block_time(),
            token_type: chain.default_asset_type(),
            transfer: Default::default(),
            token_transfer: None,
            swap: None,
        }
    }

    #[tokio::test]
    async fn test_cached_chain_returns_only_requested_chain() {
        let cacher = Arc::new(
            MemoryFeeEstimatesCacher::default()
                .with_estimates(Chain::Ethereum, cached(Chain::Ethereum), true)
                .with_estimates(Chain::Tron, cached(Chain::Tron), true),
        );
        let response = client(cacher.clone()).get_chain_fee_estimates(Chain::Tron).await.unwrap();

        assert_eq!(response.asset.id.chain, Chain::Tron);
        assert!(response.transfer.is_empty());
        assert!(cacher.writes().is_empty());
    }

    #[tokio::test]
    async fn test_expired_chain_refreshes_with_configured_duration() {
        let cacher = Arc::new(MemoryFeeEstimatesCacher::default().with_estimates(Chain::Tron, cached(Chain::Tron), false));
        let client = client(cacher.clone());
        let response = client.get_chain_fee_estimates(Chain::Tron).await.unwrap();
        client.get_chain_fee_estimates(Chain::Tron).await.unwrap();

        assert_eq!(response.asset.id.chain, Chain::Tron);
        assert_eq!(response.transfer[&FeePriority::Normal].value, "1");
        assert_eq!(cacher.writes(), vec![(Chain::Tron, Duration::from_secs(120))]);
    }

    #[tokio::test]
    async fn test_failed_refresh_returns_previous_chain_estimates() {
        let cacher = Arc::new(MemoryFeeEstimatesCacher::default().with_estimates(Chain::Ethereum, cached(Chain::Ethereum), false));
        let response = client(cacher.clone()).get_chain_fee_estimates(Chain::Ethereum).await.unwrap();

        assert_eq!(response.asset.id.chain, Chain::Ethereum);
        assert!(cacher.writes().is_empty());
    }

    #[tokio::test]
    async fn test_failed_refresh_without_previous_estimates_returns_error() {
        let cacher = Arc::new(MemoryFeeEstimatesCacher::default());

        assert!(client(cacher.clone()).get_chain_fee_estimates(Chain::Ethereum).await.is_err());
        assert!(cacher.writes().is_empty());
    }

    #[tokio::test]
    async fn test_aggregate_reads_cached_values_without_refreshing() {
        let cacher = Arc::new(
            MemoryFeeEstimatesCacher::default()
                .with_estimates(Chain::Tron, cached(Chain::Tron), false)
                .with_estimates(Chain::Bitcoin, cached(Chain::Bitcoin), false),
        );
        let response = client(cacher.clone()).get_fee_estimates().await.unwrap();

        assert_eq!(response.iter().map(|estimates| estimates.asset.id.chain).collect::<Vec<_>>(), vec![Chain::Bitcoin, Chain::Tron]);
        assert!(cacher.writes().is_empty());
    }

    #[tokio::test]
    async fn test_aggregate_empty_cache_does_not_refresh() {
        let cacher = Arc::new(MemoryFeeEstimatesCacher::default());

        assert!(client(cacher.clone()).get_fee_estimates().await.unwrap().is_empty());
        assert!(cacher.writes().is_empty());
    }

    #[test]
    fn test_map_fee_estimates() {
        let ethereum = Asset::mock();
        let estimates = TransactionFeeEstimates {
            fee_asset: ethereum.id.clone(),
            transfer: vec![TransactionFeeEstimate {
                priority: FeePriority::Normal,
                gas_price_type: GasPriceType::eip1559(51_000_000u64, 1_000_000u64),
                fee: 1_092_000_000_000u64.into(),
            }],
            token_transfer: None,
            swap: None,
        };

        let response = map_fee_estimates(ethereum.clone(), estimates, 3_520.42).unwrap();

        assert_eq!(response.asset, ethereum);
        assert_eq!(response.transfer[&FeePriority::Normal].base, "0.051");
        assert_eq!(response.transfer[&FeePriority::Normal].priority_fee, "0.001");
        assert_eq!(response.transfer[&FeePriority::Normal].value, "0.000001092");
        assert_eq!(response.transfer[&FeePriority::Normal].fiat_value, "0.00384429864");

        let estimates = TransactionFeeEstimates {
            fee_asset: Asset::mock_sol().id,
            transfer: vec![TransactionFeeEstimate {
                priority: FeePriority::Normal,
                gas_price_type: GasPriceType::solana(5_000u64, 10_000u64, 100_000u64),
                fee: 15_000u64.into(),
            }],
            token_transfer: None,
            swap: None,
        };
        let response = map_fee_estimates(Asset::mock_sol(), estimates, 150.0).unwrap();

        assert_eq!(response.transfer[&FeePriority::Normal].base, "0.000005");
        assert_eq!(response.transfer[&FeePriority::Normal].priority_fee, "0.00001");
        assert_eq!(response.transfer[&FeePriority::Normal].value, "0.000015");
    }
}
