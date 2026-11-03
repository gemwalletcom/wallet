use std::fmt::Debug;

use gem_client::{Client, ClientExt, Target};

use super::model::{BridgersResponse, QuoteData, QuoteRequest, RecordsData, RecordsRequest, SwapData, SwapRequest};
use crate::SwapperError;

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
        BridgersResponse::data(self.client.post(BridgersTarget::Quote, request).await?)
    }

    pub async fn get_swap(&self, request: &SwapRequest) -> Result<SwapData, SwapperError> {
        BridgersResponse::data(self.client.post(BridgersTarget::Swap, request).await?)
    }

    pub async fn get_records(&self, request: &RecordsRequest) -> Result<RecordsData, SwapperError> {
        BridgersResponse::data(self.client.post(BridgersTarget::Records, request).await?)
    }
}
