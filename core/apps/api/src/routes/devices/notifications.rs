use std::sync::Arc;

use axum::extract::State;
use primitives::InAppNotification;
use serde::Deserialize;
use services::notifications::NotificationsClient;

use crate::auth::device::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use crate::error::ApiError;
use crate::request::{Query, QueryLimitParam, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct NotificationsQuery {
    #[serde(default, deserialize_with = "lenient")]
    from_timestamp: Option<u64>,
    #[serde(default)]
    limit: QueryLimitParam,
}

pub async fn get_notifications(device: AuthenticatedDevice, Query(query): Query<NotificationsQuery>, State(client): State<Arc<NotificationsClient>>) -> Result<ApiResponse<Vec<InAppNotification>>, ApiError> {
    Ok(client.get_notifications(&device.record.device.id, query.from_timestamp, query.limit.0).await?.into())
}

pub async fn mark_read(device: AuthenticatedDeviceWallet, State(client): State<Arc<NotificationsClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.mark_as_read(device.wallet_id).await?.into())
}
