use super::{AlienError, AlienResponse, AlienTarget};

use async_trait::async_trait;
use gem_jsonrpc::rpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError>;
}

#[derive(Debug)]
pub struct AlienProviderWrapper {
    provider: Arc<dyn AlienProvider>,
}

impl AlienProviderWrapper {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl GenericRpcProvider for AlienProviderWrapper {
    type Error = AlienError;

    async fn request(&self, target: AlienTarget) -> Result<RpcResponse, Self::Error> {
        Ok(self.provider.request(target).await?.to_rpc_response())
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        self.provider.get_endpoint(chain)
    }
}
