use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
use async_trait::async_trait;
use primitives::Chain;

#[derive(Debug)]
pub struct TestAlienProvider {
    endpoint: String,
    response: AlienResponse,
}

impl TestAlienProvider {
    pub fn new(endpoint: impl Into<String>, response: AlienResponse) -> Self {
        Self {
            endpoint: endpoint.into(),
            response,
        }
    }

    pub fn with_status(status: u16) -> Self {
        Self::new(
            "https://example.invalid",
            AlienResponse {
                status: Some(status),
                data: Vec::new(),
            },
        )
    }
}

#[async_trait]
impl AlienProvider for TestAlienProvider {
    async fn request(&self, _target: AlienTarget) -> Result<AlienResponse, AlienError> {
        Ok(self.response.clone())
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
        Ok(self.endpoint.clone())
    }
}
