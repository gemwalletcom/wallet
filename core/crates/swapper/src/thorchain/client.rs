use super::{
    asset::THORChainAsset,
    model::{AsgardVault, InboundAddress, QuoteSwapRequest, QuoteSwapResponse, TransactionStatus},
};
use crate::{SwapperError, cache_headers};
use gem_client::{Client, ClientExt};
use primitives::duration::MINUTE;
use serde_urlencoded;
use std::fmt::Debug;

const INBOUND_ADDRESS_CACHE_TTL_SECONDS: u64 = 10 * MINUTE.as_secs();

#[derive(Clone, Debug)]
pub struct ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(
        &self,
        from_asset: THORChainAsset,
        to_asset: THORChainAsset,
        value: String,
        streaming_interval: i64,
        streaming_quantity: i64,
        affiliate: String,
        affiliate_bps: i64,
    ) -> Result<QuoteSwapResponse, SwapperError> {
        let params = QuoteSwapRequest {
            from_asset: from_asset.asset_name(),
            to_asset: to_asset.asset_name(),
            amount: value,
            affiliate,
            affiliate_bps,
            streaming_interval,
            streaming_quantity,
        };
        let query = serde_urlencoded::to_string(params).map_err(SwapperError::from)?;
        let path = format!("/thorchain/quote/swap?{query}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub async fn get_inbound_addresses(&self) -> Result<Vec<InboundAddress>, SwapperError> {
        self.client
            .get_with_headers("/thorchain/inbound_addresses", cache_headers(INBOUND_ADDRESS_CACHE_TTL_SECONDS))
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_asgard_vaults(&self) -> Result<Vec<AsgardVault>, SwapperError> {
        self.client.get("/thorchain/vaults/asgard").await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<TransactionStatus, SwapperError> {
        let path = format!("/thorchain/tx/status/{hash}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
