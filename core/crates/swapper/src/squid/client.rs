use std::fmt::Debug;

use gem_client::{Client, ClientExt};

use super::model::{SquidRouteRequest, SquidRouteResponse, SquidStatusResponse};
use crate::SwapperError;

#[derive(Clone, Debug)]
pub struct SquidClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> SquidClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_route(&self, request: &SquidRouteRequest) -> Result<SquidRouteResponse, SwapperError> {
        self.client.post("/v2/route", request).await.map_err(SwapperError::from)
    }

    pub async fn get_status(&self, transaction_hash: &str, source_chain_id: &str) -> Result<SquidStatusResponse, SwapperError> {
        let path = format!("/v2/status?transactionId={transaction_hash}&fromChainId={source_chain_id}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;
    use crate::squid::model::SquidStatus;

    #[tokio::test]
    async fn test_get_status_includes_source_chain() {
        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/v2/status?transactionId=ABC123&fromChainId=cosmoshub-4");
            Ok(include_bytes!("../../testdata/squid/status_response.json").to_vec())
        });

        let result = SquidClient::new(client).get_status("ABC123", "cosmoshub-4").await.unwrap();

        assert_eq!(result.squid_transaction_status, SquidStatus::Success);
    }
}
