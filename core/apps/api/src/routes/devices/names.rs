use std::sync::Arc;

use axum::extract::State;
use config_keys::ConfigKey;
use name_resolver::NameClient;
use primitives::name::NameRecord;
use services::ConfigCacher;

use crate::auth::device::AuthenticatedDevice;
use crate::error::ApiError;
use crate::request::{ChainQuery, Path, Query};
use crate::response::ApiResponse;

pub async fn get_name(
    _device: AuthenticatedDevice,
    Path(name): Path<String>,
    Query(query): Query<ChainQuery>,
    State(client): State<Arc<NameClient>>,
    State(config): State<Arc<ConfigCacher>>,
) -> Result<ApiResponse<Option<NameRecord>>, ApiError> {
    let max_label_length = config.get_usize(ConfigKey::NameMaxLabelLength).await?;
    Ok(client.resolve(&name, query.chain.0, max_label_length).await?.into())
}
