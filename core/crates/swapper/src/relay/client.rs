use std::fmt::Debug;

use gem_client::{Client, ClientExt};

use super::model::{RelayChainsResponse, RelayErrorResponse, RelayQuoteRequest, RelayQuoteResponse, RelayRequestsResponse};
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
            .post_or_error::<_, _, RelayErrorResponse>("/quote/v2", &request)
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_request(&self, identifier: &str) -> Result<RelayRequestsResponse, SwapperError> {
        let path = format!("/requests/v3?term={}", identifier);
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub async fn get_chains(&self) -> Result<RelayChainsResponse, SwapperError> {
        self.client.get("/chains").await.map_err(SwapperError::from)
    }
}
