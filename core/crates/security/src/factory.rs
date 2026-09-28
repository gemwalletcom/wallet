use std::error::Error;
use std::sync::Arc;

use gem_client::ReqwestClient;
use primitives::AccessTokenCacher;

use crate::config::ScanProviderConfig;
use crate::providers::{goplus::GoPlusProvider, hashdit::HashDitProvider, jupiter::JupiterProvider, tronscan::TronscanProvider};
use crate::{TokenScanProviders, TransactionScanProviders};

pub struct ScanProviderFactory;

struct ScanProviders {
    goplus: Arc<GoPlusProvider<ReqwestClient>>,
    hashdit: Arc<HashDitProvider<ReqwestClient>>,
    jupiter: Arc<JupiterProvider<ReqwestClient>>,
    tronscan: Arc<TronscanProvider<ReqwestClient>>,
}

impl ScanProviders {
    fn new(config: &ScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = gem_client::builder().timeout(config.timeout).build()?;
        let remote = |config: &gem_client::RemoteProviderConfig| config.configure_client(ReqwestClient::new(String::new(), client.clone()));
        Ok(Self {
            goplus: Arc::new(GoPlusProvider::new(
                ReqwestClient::new(config.goplus.url.clone(), client.clone()),
                &config.goplus.public_key,
                &config.goplus.secret_key,
                Some(access_token_cacher),
            )),
            hashdit: Arc::new(HashDitProvider::new(remote(&config.hashdit), &config.hashdit.key)),
            jupiter: Arc::new(JupiterProvider::new(remote(&config.jupiter), &config.jupiter.key)),
            tronscan: Arc::new(TronscanProvider::new(remote(&config.tronscan), &config.tronscan.key)),
        })
    }
}

impl ScanProviderFactory {
    pub fn new_transaction_providers(config: &ScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
        let providers = ScanProviders::new(config, access_token_cacher)?;
        Ok(TransactionScanProviders {
            addresses: vec![providers.goplus, providers.hashdit.clone(), providers.tronscan],
            poisoning: vec![providers.hashdit.clone()],
            websites: vec![providers.hashdit],
        })
    }

    pub fn new_token_providers(config: &ScanProviderConfig, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Result<TokenScanProviders, Box<dyn Error + Send + Sync>> {
        let providers = ScanProviders::new(config, access_token_cacher)?;
        Ok(vec![providers.goplus, providers.hashdit, providers.jupiter, providers.tronscan])
    }
}
