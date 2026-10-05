use std::fmt::Debug;

use gem_client::{Client, ClientExt, Target};
use serde::de::DeserializeOwned;

use super::model::{BridgersResponse, QuoteData, QuoteRequest, RecordsData, RecordsRequest, SwapData, SwapRequest};
use crate::SwapperError;

const SUCCESS_CODE: u64 = 100;
const QUOTE_FAIL_CODE: u64 = 412;

#[derive(Clone, Debug)]
enum BridgersTarget {
    Quote,
    Swap,
    Records,
}

impl Target for BridgersTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/api/sswap/quote".to_string(),
            Self::Swap => "/api/sswap/swap".to_string(),
            Self::Records => "/api/exchangeRecord/getTransData".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct BridgersClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> BridgersClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteData, SwapperError> {
        Self::data(self.client.post(BridgersTarget::Quote, request).await?)
    }

    pub async fn get_swap(&self, request: &SwapRequest) -> Result<SwapData, SwapperError> {
        Self::data(self.client.post(BridgersTarget::Swap, request).await?)
    }

    pub async fn get_records(&self, request: &RecordsRequest) -> Result<RecordsData, SwapperError> {
        Self::data(self.client.post(BridgersTarget::Records, request).await?)
    }

    fn data<T: DeserializeOwned>(response: BridgersResponse) -> Result<T, SwapperError> {
        match response.res_code {
            SUCCESS_CODE => serde_json::from_value(response.data).map_err(SwapperError::compute_quote_error),
            QUOTE_FAIL_CODE => Err(SwapperError::NoQuoteAvailable),
            _ => Err(SwapperError::ComputeQuoteError(response.res_msg)),
        }
    }
}
