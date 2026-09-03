use std::fmt::Debug;

use gem_client::{Client, ClientError, ClientExt};

use super::model::{ActionRequest, ActionResponse, PathsQuery, PathsResponse, StatusQuery, StatusResponse};
use super::target::SwapsXyzTarget;
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
        self.upstream
            .get(SwapsXyzTarget::Paths {
                query: PathsQuery::native(source_chain_id, destination_chain_id),
            })
            .await
            .map_err(Into::into)
    }

    pub async fn get_action(&self, request: &ActionRequest) -> Result<ActionResponse, SwapperError> {
        self.api.post(SwapsXyzTarget::Action, request).await.map_err(Into::into)
    }

    pub async fn get_status(&self, transaction_hash: &str, chain_id: u64) -> Result<Option<StatusResponse>, SwapperError> {
        let target = SwapsXyzTarget::Status {
            query: StatusQuery {
                tx_hash: transaction_hash.to_string(),
                chain_id,
            },
        };
        match self.upstream.get(target).await {
            Ok(response) => Ok(Some(response)),
            Err(ClientError::Http { status: 404, .. }) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
