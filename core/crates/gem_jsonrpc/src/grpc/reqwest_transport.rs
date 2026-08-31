use async_trait::async_trait;
use std::error::Error;

use super::{GrpcTransport, grpc_headers, validate_http_status};

#[derive(Clone, Debug)]
pub struct ReqwestGrpcTransport {
    client: reqwest::Client,
}

impl ReqwestGrpcTransport {
    pub fn new() -> Self {
        Self {
            client: gem_client::reqwest_client(),
        }
    }
}

impl Default for ReqwestGrpcTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrpcTransport for ReqwestGrpcTransport {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let mut request = self.client.post(format!("{}{}", endpoint.trim_end_matches('/'), path));
        for (name, value) in grpc_headers() {
            request = request.header(name, value);
        }
        let response = request.body(body).send().await?;
        let status = response.status().as_u16();
        validate_grpc_status(&response)?;
        let bytes = response.bytes().await?.to_vec();
        validate_http_status(Some(status))?;
        Ok(bytes)
    }
}

fn validate_grpc_status(response: &reqwest::Response) -> Result<(), Box<dyn Error + Send + Sync>> {
    let header = |name: &str| response.headers().get(name).and_then(|value| value.to_str().ok());
    if let Some(code) = header("grpc-status")
        && code != "0"
    {
        let message = decode_grpc_status_message(header("grpc-message").unwrap_or_default());
        return Err(format!("gRPC error {code}: {message}").into());
    }
    Ok(())
}

fn decode_grpc_status_message(value: &str) -> String {
    percent_encoding::percent_decode_str(value).decode_utf8_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_grpc_status_message() {
        assert_eq!(
            decode_grpc_status_message("Available%20amount%3A%200%20%3C%2014999840000"),
            "Available amount: 0 < 14999840000"
        );
        assert_eq!(decode_grpc_status_message("plain message"), "plain message");
        assert_eq!(decode_grpc_status_message("trailing%2"), "trailing%2");
    }
}
