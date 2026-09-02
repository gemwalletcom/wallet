use std::sync::Arc;

use primitives::Chain;

use crate::gateway::{GatewayError, GemGateway};
use crate::models::node::GemNodeStatus;

#[derive(uniffi::Object)]
pub struct GemNodeStatusService {
    gateway: Arc<GemGateway>,
}

#[uniffi::export]
impl GemNodeStatusService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>) -> Self {
        Self { gateway }
    }

    pub async fn node_status(&self, chain: Chain, url: String) -> Result<GemNodeStatus, GatewayError> {
        self.gateway.get_node_status(chain, &url).await
    }

    pub async fn check_node(&self, chain: Chain, url: String) -> Result<GemNodeStatus, GatewayError> {
        self.gateway.check_node(chain, &url).await
    }
}
