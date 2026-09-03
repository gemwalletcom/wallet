use super::model::{BuildRequest, BuildResponse};
use crate::SwapperError;
use gem_client::{Client, ClientExt, build_path_with_query};

#[derive(Clone, Debug)]
pub(super) struct JupiterClient<C>
where
    C: Client + Clone,
{
    client: C,
}

impl<C> JupiterClient<C>
where
    C: Client + Clone,
{
    pub(super) fn new(client: C) -> Self {
        Self { client }
    }

    pub(super) async fn get_build(&self, request: &BuildRequest) -> Result<BuildResponse, SwapperError> {
        let path = build_path_with_query("/swap/v2/build", request);
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
