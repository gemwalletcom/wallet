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
pub use model::{AddressTarget, ScanResult, TokenTarget};

pub type AddressScanProviders = Vec<Arc<dyn AddressScanProvider>>;
pub type TokenScanProviders = Vec<Arc<dyn TokenScanProvider>>;

#[async_trait]
pub trait AddressScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait TokenScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_chain(&self, chain: Chain) -> bool;
    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>>;
}
