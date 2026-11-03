use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::header::CACHE_CONTROL;
use primitives::config::ConfigResponse;
use serde::Deserialize;
use services::app::ConfigClient;

use crate::error::ApiError;
use crate::request::{ClientCountry, ClientIp, PlatformStoreParam, Query};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct ConfigQuery {
    store: Option<PlatformStoreParam>,
}

pub async fn get_config(ClientIp(ip): ClientIp, ClientCountry(country): ClientCountry, Query(query): Query<ConfigQuery>, State(client): State<Arc<ConfigClient>>) -> Result<Response, ApiError> {
    let config: ApiResponse<ConfigResponse> = client.get_config(&ip.to_string(), country.as_deref(), query.store.map(|store| store.0)).await?.into();
    Ok(([(CACHE_CONTROL, "private, no-store")], config).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use http::{Request, StatusCode};
    use primitives::PlatformStore;

    #[tokio::test]
    async fn test_config_query() {
        for (query, store) in [
            ("", None),
            ("?store=macDmg", Some(PlatformStore::MacDmg)),
            ("?store=appStore", Some(PlatformStore::AppStore)),
            ("?store=googlePlay", Some(PlatformStore::GooglePlay)),
        ] {
            let (mut parts, _) = Request::builder().uri(format!("/v1/config{query}")).body(()).unwrap().into_parts();
            let Query(parsed) = Query::<ConfigQuery>::from_request_parts(&mut parts, &()).await.unwrap();
            assert_eq!(parsed.store.map(|store| store.0), store);
        }

        let (mut parts, _) = Request::builder().uri("/v1/config?store=invalid").body(()).unwrap().into_parts();
        assert_eq!(
            Query::<ConfigQuery>::from_request_parts(&mut parts, &()).await.err(),
            Some(ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "store: Invalid platform store: invalid"))
        );
    }
}
