use async_trait::async_trait;
use std::error::Error;

use super::{GrpcTransport, HEADER_MESSAGE, HEADER_STATUS, grpc_headers, validate_grpc_status, validate_http_status};

#[derive(Clone, Debug)]
pub struct ReqwestGrpcTransport {
    client: reqwest::Client,
}

impl ReqwestGrpcTransport {
    pub fn new() -> Self {
        Self::new_with_client(gem_client::reqwest_client())
    }

    pub fn new_with_client(client: reqwest::Client) -> Self {
        Self { client }
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
        let header = |name: &str| response.headers().get(name).and_then(|value| value.to_str().ok());
        validate_grpc_status(header(HEADER_STATUS), header(HEADER_MESSAGE))?;
        let bytes = response.bytes().await?.to_vec();
        validate_http_status(Some(status))?;
        Ok(bytes)
    }
}
