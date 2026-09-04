use std::error::Error;
use std::sync::Arc;

use gem_client::ReqwestClient;
use primitives::AccessTokenCacher;

use crate::config::{AddressScanProviderConfig, TokenScanProviderConfig};
use crate::providers::{goplus::GoPlusProvider, hashdit::HashDitProvider, jupiter::JupiterProvider};
use crate::{AddressScanProviders, TokenScanProviders};

pub struct ScanProviderFactory;

impl ScanProviderFactory {
    pub fn new_address_providers(config: AddressScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<AddressScanProviders, Box<dyn Error + Send + Sync>> {
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

    pub fn new_token_providers(config: TokenScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<TokenScanProviders, Box<dyn Error + Send + Sync>> {
        let client = gem_client::builder().timeout(config.timeout).build()?;
        Ok(vec![
            Arc::new(GoPlusProvider::new(
                ReqwestClient::new(config.goplus.url, client.clone()),
                &config.goplus.public_key,
                &config.goplus.secret_key,
                Some(access_token_cacher),
            )),
            Arc::new(JupiterProvider::new(
                config.jupiter.configure_client(ReqwestClient::new(String::new(), client)),
                &config.jupiter.key,
            )),
        ])
    }
}
