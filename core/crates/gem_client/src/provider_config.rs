use std::time::Duration;

pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct RemoteProviderConfig {
    pub url: String,
    pub key: String,
    pub timeout: Duration,
}

impl RemoteProviderConfig {
    #[cfg(feature = "reqwest")]
    pub fn configure_client(&self, client: crate::ReqwestClient) -> crate::ReqwestClient {
        client.with_base_url(self.url.clone()).with_request_timeout(self.timeout)
    }
}

impl Default for RemoteProviderConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            key: String::new(),
            timeout: DEFAULT_REQUEST_TIMEOUT,
        }
    }
}
