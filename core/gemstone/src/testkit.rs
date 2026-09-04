use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
use async_trait::async_trait;
use primitives::Chain;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TestAlienProvider {
    endpoint: String,
    response: Arc<AlienResponse>,
    requested: Mutex<Vec<String>>,
}

impl TestAlienProvider {
    pub fn new(endpoint: impl Into<String>, response: AlienResponse) -> Self {
        Self {
            endpoint: endpoint.into(),
            response: Arc::new(response),
            requested: Mutex::new(Vec::new()),
        }
    }

    pub fn with_status(status: u16) -> Self {
        Self::new("https://example.invalid", AlienResponse::new(Some(status), Vec::new()))
    }

    pub fn with_json(status: u16, body: &str) -> Self {
        Self::new("https://example.invalid", AlienResponse::new(Some(status), body.as_bytes().to_vec()))
    }

    pub fn requested_paths(&self) -> Vec<String> {
        self.requested.lock().unwrap().clone()
    }
}

#[async_trait]
impl AlienProvider for TestAlienProvider {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
        let path = target.url.find("/v").map(|index| target.url[index..].to_string()).unwrap_or(target.url);
        self.requested.lock().unwrap().push(path);
        Ok(self.response.clone())
    }

    fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
        Ok(self.endpoint.clone())
    }
}
