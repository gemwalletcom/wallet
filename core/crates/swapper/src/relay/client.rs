use std::{collections::HashMap, fmt::Debug};

use gem_client::{CONTENT_TYPE, Client, ClientExt, ContentType};

use super::model::{RelayChainsResponse, RelayErrorResponse, RelayQuoteRequest, RelayQuoteResponse, RelayRequestsResponse};
use crate::{SwapperError, error::ProviderErrorResponse};

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
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().into())]);
        self.client.post_with("/quote/v2", &request, headers).await.map_err(RelayErrorResponse::map_client_error)
    }

    pub async fn get_request(&self, identifier: &str) -> Result<RelayRequestsResponse, SwapperError> {
        let path = format!("/requests/v3?term={}", identifier);
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub async fn get_chains(&self) -> Result<RelayChainsResponse, SwapperError> {
        self.client.get("/chains").await.map_err(SwapperError::from)
    }
}
