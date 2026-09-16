use super::{AlienError, AlienResponse, AlienTarget};
use crate::services::preferences::GemPreferencesStore;

use async_trait::async_trait;
use gem_jsonrpc::rpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError>;
}

pub trait NodeEndpoints: Send + Sync + Debug {
    fn node_url(&self, chain: Chain) -> Result<String, AlienError>;
}

pub struct PreferencesNodeEndpoints {
    preferences: Arc<dyn GemPreferencesStore>,
}

impl PreferencesNodeEndpoints {
    pub fn new(preferences: Arc<dyn GemPreferencesStore>) -> Self {
        Self { preferences }
    }
}

impl Debug for PreferencesNodeEndpoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreferencesNodeEndpoints").finish()
    }
}

impl NodeEndpoints for PreferencesNodeEndpoints {
    fn node_url(&self, chain: Chain) -> Result<String, AlienError> {
        Ok(crate::services::node::node_url(self.preferences.as_ref(), chain))
    }
}

#[derive(Debug)]
pub struct AlienProviderWrapper {
    provider: Arc<dyn AlienProvider>,
    endpoints: Option<Arc<dyn NodeEndpoints>>,
}

impl AlienProviderWrapper {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider, endpoints: None }
    }

    pub fn with_endpoints(provider: Arc<dyn AlienProvider>, endpoints: Arc<dyn NodeEndpoints>) -> Self {
        Self {
            provider,
            endpoints: Some(endpoints),
        }
    }
}

#[async_trait]
impl GenericRpcProvider for AlienProviderWrapper {
    type Error = AlienError;

    async fn request(&self, target: AlienTarget) -> Result<RpcResponse, Self::Error> {
        Ok(self.provider.request(target).await?.to_rpc_response())
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        match &self.endpoints {
            Some(endpoints) => endpoints.node_url(chain),
            None => Err(AlienError::RequestError {
                msg: format!("no node endpoint for {chain}"),
            }),
        }
    }
}
