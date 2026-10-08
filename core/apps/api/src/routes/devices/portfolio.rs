use std::sync::Arc;

use axum::extract::State;
use primitives::{PortfolioAssets, PortfolioAssetsRequest};
use serde::Deserialize;
use services::prices::PortfolioClient;

use crate::auth::device::{AuthenticatedDevice, DeviceJson};
use crate::error::ApiError;
use crate::request::{ChartPeriodParam, Query};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct PeriodQuery {
    period: ChartPeriodParam,
}

pub async fn get_portfolio_assets(_device: AuthenticatedDevice, Query(query): Query<PeriodQuery>, State(client): State<Arc<PortfolioClient>>, request: DeviceJson<PortfolioAssetsRequest>) -> Result<ApiResponse<PortfolioAssets>, ApiError> {
    Ok(client.get_portfolio_charts(request.into_inner().assets, query.period.0).await?.into())
}
