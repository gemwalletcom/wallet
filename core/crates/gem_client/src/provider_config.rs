#[derive(Clone, Default)]
pub struct RemoteProviderConfig {
    pub url: String,
    pub key: String,
}

impl RemoteProviderConfig {
    #[cfg(feature = "reqwest")]
    pub fn configure_client(&self, client: crate::ReqwestClient) -> crate::ReqwestClient {
        client.with_base_url(self.url.clone())
    }
}
