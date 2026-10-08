use super::{AlienError, AlienResponse, AlienTarget};
use crate::services::node::GemNodeService;

use async_trait::async_trait;
use gem_jsonrpc::alien::RpcProvider;
use gem_jsonrpc::rpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError>;
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
}

#[derive(Debug)]
pub struct AlienRpcProvider {
    transport: AlienProviderWrapper,
    nodes: Arc<GemNodeService>,
}

impl AlienRpcProvider {
    pub fn new(provider: Arc<dyn AlienProvider>, nodes: Arc<GemNodeService>) -> Self {
        Self {
            transport: AlienProviderWrapper::new(provider),
            nodes,
        }
    }
}

#[async_trait]
impl GenericRpcProvider for AlienRpcProvider {
    type Error = AlienError;

    async fn request(&self, target: AlienTarget) -> Result<RpcResponse, Self::Error> {
        self.transport.request(target).await
    }
}

impl RpcProvider for AlienRpcProvider {
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        Ok(self.nodes.node_url(chain))
    }
}
