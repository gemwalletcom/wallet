use std::{collections::HashMap, fmt::Debug};

use gem_client::{CONTENT_TYPE, Client, ClientError, ClientExt, ContentType};

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
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().into())]);
        self.client.post_with("/quote/v2", &request, headers).await.map_err(map_quote_error)
    }

    pub async fn get_request(&self, identifier: &str) -> Result<RelayRequestsResponse, SwapperError> {
        let path = format!("/requests/v3?term={}", identifier);
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub async fn get_chains(&self) -> Result<RelayChainsResponse, SwapperError> {
        self.client.get("/chains").await.map_err(SwapperError::from)
    }
}

fn map_quote_error(error: ClientError) -> SwapperError {
    if let ClientError::Http { body, .. } = &error
        && let Ok(response) = serde_json::from_slice::<RelayErrorResponse>(body)
        && let Some(swapper_error) = response.into_swapper_error()
    {
        return swapper_error;
    }
    SwapperError::from(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_quote_error() {
        let error = ClientError::Http {
            status: 400,
            body: br#"{"message":"Swap output amount is too small to cover fees required to execute swap","errorCode":"AMOUNT_TOO_LOW"}"#.to_vec(),
        };
        assert_eq!(map_quote_error(error), SwapperError::InputAmountError { min_amount: None });
    }
}
