use std::fmt::Debug;

use gem_client::{Client, ClientError, ClientExt};

use super::model::{ActionRequest, ActionResponse, PathsResponse, StatusResponse};
use crate::SwapperError;

#[derive(Clone, Debug)]
pub(super) struct SwapsXyzClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    upstream: C,
    api: C,
}

impl<C> SwapsXyzClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(upstream: C, api: C) -> Self {
        Self { upstream, api }
    }

    pub async fn get_paths(&self, source_chain_id: u64, destination_chain_id: u64) -> Result<PathsResponse, SwapperError> {
        let query = serde_urlencoded::to_string([
            ("srcChainId", source_chain_id.to_string()),
            ("srcToken", super::NATIVE_TOKEN.to_string()),
            ("dstChainId", destination_chain_id.to_string()),
        ])?;
        self.upstream.get(&format!("/getPaths?{query}")).await.map_err(Into::into)
    }

    pub async fn get_action(&self, request: &ActionRequest) -> Result<ActionResponse, SwapperError> {
        self.api.post("/action", request).await.map_err(Into::into)
    }

    pub async fn get_status(&self, transaction_hash: &str, chain_id: u64) -> Result<Option<StatusResponse>, SwapperError> {
        let query = serde_urlencoded::to_string([("txHash", transaction_hash.to_string()), ("chainId", chain_id.to_string())])?;
        match self.upstream.get(&format!("/getStatus?{query}")).await {
            Ok(response) => Ok(Some(response)),
            Err(ClientError::Http { status: 404, .. }) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
