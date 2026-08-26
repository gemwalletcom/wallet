use async_trait::async_trait;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use crate::{
    alien,
    rpc::{HttpMethod, Target},
};

#[cfg(feature = "reqwest")]
mod reqwest_transport;
#[cfg(feature = "reqwest")]
pub use reqwest_transport::ReqwestGrpcTransport;

#[async_trait]
pub trait GrpcTransport: Send + Sync + fmt::Debug {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>;
}

fn unary_target(endpoint: &str, path: &str, body: Vec<u8>) -> Target {
    Target {
        url: format!("{}{}", endpoint.trim_end_matches('/'), path),
        method: HttpMethod::Post,
        headers: Some(grpc_headers()),
        body: Some(body),
    }
}

fn validate_http_status(status: Option<u16>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(status) = status
        && !(200..300).contains(&status)
    {
        return Err(format!("gRPC HTTP error: status {status}").into());
    }
    Ok(())
}

fn grpc_headers() -> HashMap<String, String> {
    HashMap::from([
        ("Content-Type".into(), "application/grpc+proto".into()),
        ("Accept".into(), "application/grpc+proto".into()),
        ("TE".into(), "trailers".into()),
        ("grpc-accept-encoding".into(), "identity".into()),
    ])
}

#[derive(Clone)]
pub struct AlienGrpcTransport {
    provider: Arc<dyn alien::RpcProvider>,
}

impl AlienGrpcTransport {
    pub fn new(provider: Arc<dyn alien::RpcProvider>) -> Self {
        Self { provider }
    }
}

impl fmt::Debug for AlienGrpcTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlienGrpcTransport").finish_non_exhaustive()
    }
}

#[async_trait]
impl GrpcTransport for AlienGrpcTransport {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let response = self.provider.request(unary_target(endpoint, path, body)).await?;
        validate_http_status(response.status)?;
        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unary_target_accepts_identity_encoding() {
        let target = unary_target("https://example.com", "/Service/Method", Vec::new());

        assert_eq!(target.headers.unwrap().get("grpc-accept-encoding").map(String::as_str), Some("identity"));
    }
}
