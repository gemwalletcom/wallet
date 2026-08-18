use super::{AlienError, AlienProvider, AlienResponse, AlienTarget};
use async_trait::async_trait;
use gem_jsonrpc::RpcProvider as GenericRpcProvider;
use primitives::Chain;
use std::sync::Arc;

pub use swapper::NativeProvider;

#[async_trait]
impl AlienProvider for NativeProvider {
    async fn request(&self, target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
        let response = <Self as GenericRpcProvider>::request(self, target).await?;
        Ok(Arc::new(AlienResponse::new(response.status, response.data)))
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        <Self as GenericRpcProvider>::get_endpoint(self, chain)
    }
}
