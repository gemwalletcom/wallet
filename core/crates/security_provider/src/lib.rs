use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::{Chain, ScanProvider};

mod config;
mod factory;
pub mod model;
pub mod providers;

pub use config::{AddressScanProviderConfig, ScanProviderRemoteConfig, TokenScanProviderConfig};
pub use factory::ScanProviderFactory;
pub use model::{AddressPoisoningTarget, AddressTarget, ScanResult, TokenTarget, WebsiteTarget};

pub type AddressScanProviders = Vec<Arc<dyn AddressScanProvider>>;
pub type AddressPoisoningProviders = Vec<Arc<dyn AddressPoisoningProvider>>;
pub type TokenScanProviders = Vec<Arc<dyn TokenScanProvider>>;
pub type WebsiteScanProviders = Vec<Arc<dyn WebsiteScanProvider>>;

#[derive(Clone)]
pub struct TransactionScanProviders {
    pub addresses: AddressScanProviders,
    pub poisoning: AddressPoisoningProviders,
    pub websites: WebsiteScanProviders,
}

impl TransactionScanProviders {
    pub fn filter_enabled(&self, enabled: &[ScanProvider]) -> Self {
        Self {
            addresses: self.addresses.iter().filter(|provider| enabled.contains(&provider.provider())).cloned().collect(),
            poisoning: self.poisoning.iter().filter(|provider| enabled.contains(&provider.provider())).cloned().collect(),
            websites: self.websites.iter().filter(|provider| enabled.contains(&provider.provider())).cloned().collect(),
        }
    }
}

#[async_trait]
pub trait AddressScanProvider: Send + Sync {
    fn provider(&self) -> ScanProvider;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait AddressPoisoningProvider: Send + Sync {
    fn provider(&self) -> ScanProvider;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_address_poisoning(&self, target: &AddressPoisoningTarget) -> Result<ScanResult<AddressPoisoningTarget>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait TokenScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait WebsiteScanProvider: Send + Sync {
    fn provider(&self) -> ScanProvider;
    async fn scan_website(&self, target: &WebsiteTarget) -> Result<ScanResult<WebsiteTarget>, Box<dyn Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;
    use crate::providers::{goplus::GoPlusProvider, hashdit::HashDitProvider, tronscan::TronscanProvider};

    #[test]
    fn test_filter_enabled() {
        let hashdit = Arc::new(HashDitProvider::new(MockClient::new(), ""));
        let providers = TransactionScanProviders {
            addresses: vec![Arc::new(GoPlusProvider::mock(MockClient::new())), hashdit.clone(), Arc::new(TronscanProvider::new(MockClient::new(), ""))],
            poisoning: vec![hashdit.clone()],
            websites: vec![hashdit],
        };
        for (enabled, addresses, hashdit_enabled) in [
            (vec![ScanProvider::GoPlus, ScanProvider::Tronscan], vec![ScanProvider::GoPlus, ScanProvider::Tronscan], false),
            (vec![ScanProvider::HashDit], vec![ScanProvider::HashDit], true),
            (vec![], vec![], false),
            (ScanProvider::all(), ScanProvider::all(), true),
        ] {
            let filtered = providers.filter_enabled(&enabled);
            assert_eq!(filtered.addresses.iter().map(|provider| provider.provider()).collect::<Vec<_>>(), addresses);
            assert_eq!(filtered.poisoning.len(), usize::from(hashdit_enabled));
            assert_eq!(filtered.websites.len(), usize::from(hashdit_enabled));
        }
    }
}
