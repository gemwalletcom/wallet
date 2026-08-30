use async_trait::async_trait;
use primitives::Chain;
use std::result::Result;

pub mod model;
pub mod providers;

pub use model::{AddressTarget, ScanResult, TokenTarget};

#[async_trait]
pub trait ScanProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn supports_address_chain(&self, chain: Chain) -> bool;
    fn supports_token_chain(&self, chain: Chain) -> bool;
    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>>;
    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>>;
}
