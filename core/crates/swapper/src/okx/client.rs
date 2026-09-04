use super::{
    auth::build_headers,
    model::{OkxClientConfig, QuoteParams, SwapParams},
    target::OkxTarget,
};
use crate::SwapperError;
use chrono::{SecondsFormat, Utc};
use gem_client::{Client, ClientExt, Target};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub(super) struct OkxDexClient<C> {
    client: C,
    config: OkxClientConfig,
}

impl<C> OkxDexClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C, config: OkxClientConfig) -> Self {
        Self { client, config }
    }

    pub async fn quote<R>(&self, params: &QuoteParams) -> Result<R, SwapperError>
    where
        R: serde::de::DeserializeOwned + Send,
    {
        self.signed_get(OkxTarget::Quote { params: params.clone() }).await
    }

    pub async fn swap<R>(&self, params: &SwapParams) -> Result<R, SwapperError>
    where
        R: serde::de::DeserializeOwned + Send,
    {
        self.signed_get(OkxTarget::Swap { params: params.clone() }).await
    }

    async fn signed_get<R>(&self, target: OkxTarget) -> Result<R, SwapperError>
    where
        R: serde::de::DeserializeOwned + Send,
    {
        let path = target.path();
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let headers = build_headers(&self.config, &timestamp, &path);
        self.client.get(&path).headers(headers).await.map_err(SwapperError::from)
    }
}
