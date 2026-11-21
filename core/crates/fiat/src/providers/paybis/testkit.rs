use gem_reqwest::ReqwestClient;

use super::client::PaybisClient;

impl PaybisClient {
    pub fn mock() -> Self {
        Self::new(ReqwestClient::new(String::new(), gem_reqwest::reqwest_client()), String::new(), String::new())
    }
}
