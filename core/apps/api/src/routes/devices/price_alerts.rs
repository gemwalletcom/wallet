use std::sync::Arc;

use axum::extract::State;
use primitives::PriceAlerts;
use serde::Deserialize;
use services::prices::PriceAlertClient;

use crate::auth::device::{AuthenticatedDevice, DeviceJson};
use crate::error::ApiError;
use crate::request::{AssetIdParam, Query, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct PriceAlertsQuery {
    #[serde(default, deserialize_with = "lenient")]
    asset_id: Option<AssetIdParam>,
}

pub async fn get_price_alerts(device: AuthenticatedDevice, Query(query): Query<PriceAlertsQuery>, State(client): State<Arc<PriceAlertClient>>) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.get_price_alerts(&device.record.device.id, query.asset_id.as_ref().map(|asset_id| &asset_id.0)).await?.into())
}

pub async fn add_price_alerts(device: AuthenticatedDevice, State(client): State<Arc<PriceAlertClient>>, price_alerts: DeviceJson<PriceAlerts>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.add_price_alerts(&device.record.device.id, price_alerts.into_inner()).await?.into())
}

pub async fn delete_price_alerts(device: AuthenticatedDevice, State(client): State<Arc<PriceAlertClient>>, price_alerts: DeviceJson<PriceAlerts>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.delete_price_alerts(&device.record.device.id, price_alerts.into_inner()).await?.into())
}
