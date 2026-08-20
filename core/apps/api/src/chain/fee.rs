use std::collections::BTreeMap;
use std::error::Error;

use cacher::{CacheKey, CacherClient};
use number_formatter::{BigNumberFormatter, CryptoFiatConverter};
use pricer::PriceClient;
use primitives::{Asset, Chain, FeePriority, FeeUnitType};
use rocket::{State, get};
use serde::{Deserialize, Serialize};
use settings_chain::{TransactionFeeEstimate, TransactionFeeEstimates};
use strum::IntoEnumIterator;

use crate::api_clients::PermissionChainRead;
use crate::assets::AssetsClient;
use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};

use super::ChainClient;

type EstimatesByPriority = BTreeMap<FeePriority, FeeEstimate>;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainFeeEstimates {
    asset: Asset,
    rate_unit: FeeUnitType,
    transfer: EstimatesByPriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_transfer: Option<EstimatesByPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    swap: Option<EstimatesByPriority>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FeeEstimate {
    base: String,
    priority_fee: String,
    value: String,
    fiat_value: String,
}

#[get("/chain/fee-estimates/<chain>")]
pub async fn get_chain_fee_estimates(
    _permission: PermissionChainRead,
    chain: ChainParam,
    chain_client: &State<ChainClient>,
    assets_client: &State<AssetsClient>,
    price_client: &State<PriceClient>,
    cacher: &State<CacherClient>,
) -> Result<ApiResponse<ChainFeeEstimates>, ApiError> {
    let chain = chain.0;
    let (cached, fresh) = futures::try_join!(
        cacher.get_cached_optional::<ChainFeeEstimates>(CacheKey::TransactionFeeEstimates(chain.as_ref())),
        cacher.get_cached_optional::<()>(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref())),
    )?;
    if let (Some(estimates), Some(())) = (cached, fresh) {
        return Ok(estimates.into());
    }

    let estimates = chain_client.get_transaction_fee_estimates(chain).await?;
    let asset = assets_client.get_asset(&estimates.fee_asset)?;
    let price = price_client.get_cache_price(&estimates.fee_asset).await?;
    let estimates = map_fee_estimates(asset, estimates, price.price.price)?;
    cacher.set_cached(CacheKey::TransactionFeeEstimates(chain.as_ref()), &estimates).await?;
    cacher.set_cached(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref()), &()).await?;
    Ok(estimates.into())
}

#[get("/chain/fee-estimates")]
pub async fn get_fee_estimates(cacher: &State<CacherClient>) -> Result<ApiResponse<Vec<ChainFeeEstimates>>, ApiError> {
    let keys = Chain::iter().map(|chain| CacheKey::TransactionFeeEstimates(chain.as_ref()).key()).collect();
    let estimates = cacher.get_values::<Vec<ChainFeeEstimates>, ChainFeeEstimates>(keys).await?;
    Ok(estimates.into())
}

fn map_fee_estimates(asset: Asset, estimates: TransactionFeeEstimates, price_usd: f64) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
    let rate_unit = asset.chain().fee_unit_type();
    let asset_decimals = asset.decimals;
    let rate_decimals = match rate_unit {
        FeeUnitType::Native => asset_decimals,
        FeeUnitType::SatVb | FeeUnitType::Gwei => rate_unit.decimals() as i32,
    };
    let map_estimates = |estimates| map_estimates_by_priority(estimates, rate_decimals, asset_decimals, price_usd);
    Ok(ChainFeeEstimates {
        transfer: map_estimates(estimates.transfer)?,
        token_transfer: estimates.token_transfer.map(&map_estimates).transpose()?,
        swap: estimates.swap.map(map_estimates).transpose()?,
        asset,
        rate_unit,
    })
}

fn map_estimates_by_priority(
    estimates: Vec<TransactionFeeEstimate>,
    rate_decimals: i32,
    asset_decimals: i32,
    price_usd: f64,
) -> Result<EstimatesByPriority, Box<dyn Error + Send + Sync>> {
    estimates
        .into_iter()
        .map(|estimate| {
            let value = estimate.fee.to_string();
            Ok((
                estimate.priority,
                FeeEstimate {
                    base: BigNumberFormatter::value(&estimate.gas_price_type.gas_price().to_string(), rate_decimals)?,
                    priority_fee: BigNumberFormatter::value(&estimate.gas_price_type.priority_fee().to_string(), rate_decimals)?,
                    value: BigNumberFormatter::value(&value, asset_decimals)?,
                    fiat_value: CryptoFiatConverter::to_fiat(&value, asset_decimals as u32, price_usd)?,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, FeePriority, GasPriceType};
    use settings_chain::{TransactionFeeEstimate, TransactionFeeEstimates};

    use super::map_fee_estimates;

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
