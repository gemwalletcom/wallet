use async_trait::async_trait;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use crate::{
    alien,
    rpc::{HttpMethod, Target},
};

const STATUS_OK: &str = "0";
const STATUS_NOT_FOUND: &str = "5";
pub const HEADER_STATUS: &str = "grpc-status";
pub const HEADER_MESSAGE: &str = "grpc-message";

#[derive(Debug, Clone, PartialEq)]
pub struct GrpcStatusError {
    pub code: String,
    pub message: String,
}

impl GrpcStatusError {
    pub fn is_not_found(&self) -> bool {
        self.code == STATUS_NOT_FOUND
    }
}

impl fmt::Display for GrpcStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gRPC error {}: {}", self.code, self.message)
    }
}

impl Error for GrpcStatusError {}

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

pub fn validate_grpc_status(status: Option<&str>, message: Option<&str>) -> Result<(), Box<dyn Error + Send + Sync>> {
    match status {
        Some(code) if code != STATUS_OK => Err(Box::new(GrpcStatusError {
            code: code.to_string(),
            message: gem_encoding::percent_decode_lossy(message.unwrap_or_default()),
        })),
        Some(_) | None => Ok(()),
    }
}

pub fn validate_http_status(status: Option<u16>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(status) = status
        && !(200..300).contains(&status)
    {
        return Err(format!("gRPC HTTP error: status {status}").into());
    }
    Ok(())
}

pub fn grpc_headers() -> HashMap<String, String> {
    HashMap::from([
        ("Content-Type".into(), "application/grpc+proto".into()),
        ("Accept".into(), "application/grpc+proto".into()),
        ("TE".into(), "trailers".into()),
        ("grpc-accept-encoding".into(), "identity".into()),
    ])
}

#[derive(Clone)]
pub struct AlienGrpcTransport {
    provider: Arc<dyn alien::RpcTransport>,
}

impl AlienGrpcTransport {
    pub fn new(provider: Arc<dyn alien::RpcTransport>) -> Self {
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
        let header = |name: &str| response.headers.get(name).map(String::as_str);
        validate_grpc_status(header(HEADER_STATUS), header(HEADER_MESSAGE))?;
        validate_http_status(response.status)?;
        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_grpc_status() {
        let error = validate_grpc_status(Some("3"), Some("Available%20amount%3A%200%20%3C%2014999840000")).unwrap_err();

        assert_eq!(
            error.downcast_ref::<GrpcStatusError>(),
            Some(&GrpcStatusError {
                code: "3".to_string(),
                message: "Available amount: 0 < 14999840000".to_string(),
            })
        );
        assert_eq!(validate_grpc_status(Some("5"), Some("trailing%2")).unwrap_err().to_string(), "gRPC error 5: trailing%2");
        assert!(validate_grpc_status(Some("0"), None).is_ok());
        assert!(validate_grpc_status(None, None).is_ok());
    }

    #[test]
    fn test_unary_target_accepts_identity_encoding() {
        let target = unary_target("https://example.com", "/Service/Method", Vec::new());

        assert_eq!(target.headers.unwrap().get("grpc-accept-encoding").map(String::as_str), Some("identity"));
    }
}
