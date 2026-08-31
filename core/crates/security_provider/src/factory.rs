use std::error::Error;
use std::sync::Arc;

use gem_client::ReqwestClient;
use primitives::AccessTokenCacher;

use crate::ScanProviders;
use crate::config::ScanProviderConfig;
use crate::providers::{goplus::GoPlusProvider, hashdit::HashDitProvider};

pub struct ScanProviderFactory;

impl ScanProviderFactory {
    pub fn new_providers(config: ScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<ScanProviders, Box<dyn Error + Send + Sync>> {
        let client = gem_client::builder().timeout(config.timeout).build()?;
        Ok(vec![
            Arc::new(GoPlusProvider::new(
                ReqwestClient::new(config.goplus.url, client.clone()),
                &config.goplus.public_key,
                &config.goplus.secret_key,
                Some(access_token_cacher),
            )),
            Arc::new(HashDitProvider::new(
                ReqwestClient::new(config.hashdit.url, client),
                &config.hashdit.public_key,
                &config.hashdit.secret_key,
            )),
        ])
    }
}
