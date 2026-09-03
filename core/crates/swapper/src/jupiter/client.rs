use super::model::{BuildRequest, BuildResponse};
use super::target::JupiterTarget;
use crate::SwapperError;
use gem_client::{Client, ClientExt};

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
        self.client.get(JupiterTarget::Build).query(request).await.map_err(SwapperError::from)
    }
}
