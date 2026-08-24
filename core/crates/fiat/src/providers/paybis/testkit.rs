use gem_client::ReqwestClient;

use super::client::PaybisClient;

impl PaybisClient {
    pub fn mock() -> Self {
        Self::new(ReqwestClient::new(String::new(), gem_client::reqwest_client()), String::new(), String::new())
    }
}
