use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::Chain;

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

#[async_trait]
pub trait AddressScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait AddressPoisoningProvider: Send + Sync {
    fn name(&self) -> &'static str;
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
    fn name(&self) -> &'static str;
    async fn scan_website(&self, target: &WebsiteTarget) -> Result<ScanResult<WebsiteTarget>, Box<dyn Error + Send + Sync>>;
}
