use std::sync::Arc;

use axum::extract::State;
use primitives::{ChainFeeEstimates, ChartPeriod, FeeEstimateChartValue};
use services::chain::FeeEstimatesClient;

use crate::error::ApiError;
use crate::request::{ChainParam, ChartPeriodQuery, Path, Query};
use crate::response::ApiResponse;

pub async fn get_fee_estimates(State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<Vec<ChainFeeEstimates>>, ApiError> {
    Ok(client.get_fee_estimates().await?.into())
}

pub async fn get_chain_fee_estimates(Path(chain): Path<ChainParam>, State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<ChainFeeEstimates>, ApiError> {
    Ok(client.get_chain_fee_estimates(chain.0).await?.into())
}

pub async fn get_chain_fee_estimates_charts(Path(chain): Path<ChainParam>, Query(query): Query<ChartPeriodQuery>, State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<Vec<FeeEstimateChartValue>>, ApiError> {
    let period = query.period.map(|period| period.0).unwrap_or(ChartPeriod::Day);
    Ok(client.get_chain_fee_estimates_charts(chain.0, period).await?.into())
}
