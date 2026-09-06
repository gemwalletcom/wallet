use std::fmt::Debug;

use gem_client::{Client, ClientExt};

use super::model::{RelayChainsResponse, RelayErrorResponse, RelayQuoteRequest, RelayQuoteResponse, RelayRequestsResponse};
use super::target::RelayTarget;
use crate::SwapperError;

#[derive(Clone, Debug)]
pub struct RelayClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> RelayClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: RelayQuoteRequest) -> Result<RelayQuoteResponse, SwapperError> {
        self.client
            .post_or_error::<_, _, RelayErrorResponse>(RelayTarget::Quote, &request)
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_request(&self, identifier: &str) -> Result<RelayRequestsResponse, SwapperError> {
        self.client.get(RelayTarget::Request { term: identifier.to_string() }).await.map_err(SwapperError::from)
    }

    pub async fn get_requests(&self, user: &str, origin_chain_id: u64) -> Result<RelayRequestsResponse, SwapperError> {
        let target = RelayTarget::Requests {
            user: user.to_string(),
            origin_chain_id,
        };
        self.client.get(target).await.map_err(SwapperError::from)
    }

    pub async fn get_chains(&self) -> Result<RelayChainsResponse, SwapperError> {
        self.client.get(RelayTarget::Chains).await.map_err(SwapperError::from)
    }
}
