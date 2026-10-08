use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use http::request::Parts;

use crate::error::ApiError;

const REAL_IP_HEADER: &str = "x-real-ip";

pub struct ClientIp(pub IpAddr);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(ip) = parts.headers.get(REAL_IP_HEADER).and_then(|value| value.to_str().ok()).and_then(|value| value.trim().parse::<IpAddr>().ok()) {
            return Ok(Self(ip));
        }
        match ConnectInfo::<SocketAddr>::from_request_parts(parts, state).await {
            Ok(ConnectInfo(address)) => Ok(Self(address.ip())),
            Err(_) => Err(ApiError::Internal("Client address is not available".to_string())),
        }
    }
}
