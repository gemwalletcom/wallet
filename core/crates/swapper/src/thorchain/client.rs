use super::{
    THORChainNetwork,
    asset::THORChainAsset,
    model::{AsgardVault, ErrorResponse, InboundAddress, QuoteSwapRequest, QuoteSwapResponse, TransactionStatus},
    target::ThorChainTarget,
};
use crate::SwapperError;
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
    network: THORChainNetwork,
}

impl<C> ThorChainSwapClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C, network: THORChainNetwork) -> Self {
        Self { client, network }
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
        let request = QuoteSwapRequest {
            from_asset: from_asset.quote_asset_name(),
            to_asset: to_asset.quote_asset_name(),
            amount: value,
            affiliate,
            affiliate_bps,
            streaming_interval,
            streaming_quantity,
        };
        self.client
            .get_or_error::<_, ErrorResponse>(ThorChainTarget::Quote { network: self.network, request })
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_inbound_addresses(&self) -> Result<Vec<InboundAddress>, SwapperError> {
        self.client
            .get(ThorChainTarget::InboundAddresses { network: self.network })
            .await
            .map_err(SwapperError::from)
    }

    pub async fn get_asgard_vaults(&self) -> Result<Vec<AsgardVault>, SwapperError> {
        self.client.get(ThorChainTarget::AsgardVaults { network: self.network }).await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<TransactionStatus, SwapperError> {
        self.client
            .get(ThorChainTarget::TransactionStatus {
                network: self.network,
                hash: hash.to_string(),
            })
            .await
            .map_err(SwapperError::from)
    }
}
