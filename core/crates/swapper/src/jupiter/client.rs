use super::model::*;
use gem_client::{Client, ClientError, ClientExt};

#[derive(Clone, Debug)]
pub struct JupiterClient<C>
where
    C: Client + Clone,
{
    client: C,
}

impl<C> JupiterClient<C>
where
    C: Client + Clone,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_build(&self, request: BuildRequest) -> Result<BuildResponse, ClientError> {
        let query_string = serde_urlencoded::to_string(&request).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let path = format!("/swap/v2/build?{}", query_string);
        self.client.get(&path).await
    }
}
