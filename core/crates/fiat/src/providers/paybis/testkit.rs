use super::client::PaybisClient;

impl PaybisClient {
    pub fn mock() -> Self {
        Self::new(gem_client::reqwest_client(), String::new(), String::new())
    }
}
