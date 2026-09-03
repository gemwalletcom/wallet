use super::{
    auth::{build_headers, build_query_string},
    model::{OkxClientConfig, QuoteParams, SwapParams},
};
use crate::SwapperError;
use chrono::{SecondsFormat, Utc};
use gem_client::{Client, ClientExt};
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
        self.signed_get("/api/v6/dex/aggregator/quote", params).await
    }

    pub async fn swap<R>(&self, params: &SwapParams) -> Result<R, SwapperError>
    where
        R: serde::de::DeserializeOwned + Send,
    {
        self.signed_get("/api/v6/dex/aggregator/swap", params).await
    }

    async fn signed_get<P, R>(&self, path: &str, params: &P) -> Result<R, SwapperError>
    where
        P: serde::Serialize,
        R: serde::de::DeserializeOwned + Send,
    {
        let query = build_query_string(params)?;
        let full_path = format!("{path}{query}");
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let headers = build_headers(&self.config, &timestamp, &full_path);
        self.client.get(&full_path).headers(headers).await.map_err(SwapperError::from)
    }
}
