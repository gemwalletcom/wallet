use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::Chain;

mod config;
mod factory;
pub mod model;
pub mod providers;

pub use config::{ScanProviderConfig, ScanProviderRemoteConfig};
pub use factory::ScanProviderFactory;
pub use model::{AddressTarget, ScanResult, TokenTarget};

pub type ScanProviders = Vec<Arc<dyn ScanProvider>>;

#[async_trait]
pub trait ScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_address_chain(&self, chain: Chain) -> bool;
    fn supports_token_chain(&self, chain: Chain) -> bool;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn Error + Send + Sync>>;
    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>>;
}
