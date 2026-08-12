use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
use async_trait::async_trait;
use primitives::Chain;
use std::sync::Arc;

#[derive(Debug)]
pub struct TestAlienProvider {
    endpoint: String,
    response: Arc<AlienResponse>,
}

impl TestAlienProvider {
    pub fn new(endpoint: impl Into<String>, response: AlienResponse) -> Self {
        Self {
            endpoint: endpoint.into(),
            response: Arc::new(response),
        }
    }

    pub fn with_status(status: u16) -> Self {
        Self::new("https://example.invalid", AlienResponse::new(Some(status), Vec::new()))
    }
}

#[async_trait]
impl AlienProvider for TestAlienProvider {
    async fn request(&self, _target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
        Ok(self.response.clone())
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
        Ok(self.endpoint.clone())
    }
}
