use super::status::StatusByTransactionHashRequest;
use super::{
    AssetsResponse, BrokerStatusResponse, DepositChannelRequest, DepositChannelResponse, QuoteRequest, QuoteResponse, VaultSwapExtras, VaultSwapResponse,
    jsonrpc::RequestSwapParameterEncoding,
    model::{ChainflipAsset, DcaParameters},
    target::BrokerTarget,
};
use crate::SwapperError;
use gem_client::{Client, ClientExt};
use gem_jsonrpc::types::{JsonRpcResult, ToJsonRpcRequest};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
    deposit_channel_client: C,
}

impl<C> BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C, deposit_channel_client: C) -> Self {
        Self { client, deposit_channel_client }
    }

    pub async fn get_assets(&self) -> Result<AssetsResponse, SwapperError> {
        self.client.get(BrokerTarget::Assets).await.map_err(SwapperError::from)
    }

    pub async fn get_quotes(&self, request: &QuoteRequest) -> Result<Vec<QuoteResponse>, SwapperError> {
        self.client.get(BrokerTarget::Quotes).query(request).await.map_err(SwapperError::from)
    }

    pub async fn encode_vault_swap(
        &self,
        source_asset: ChainflipAsset,
        destination_asset: ChainflipAsset,
        destination_address: String,
        broker_commission: u32,
        boost_fee: Option<u32>,
        extra_params: VaultSwapExtras,
        dca_params: Option<DcaParameters>,
    ) -> Result<VaultSwapResponse, SwapperError> {
        let request = RequestSwapParameterEncoding {
            source_asset,
            destination_asset,
            destination_address,
            broker_commission,
            extra_params,
            boost_fee,
            dca_params,
        }
        .to_jsonrpc_request(1);
        let response: JsonRpcResult<VaultSwapResponse> = self.client.post(BrokerTarget::Rpc, &request).await?;
        response.take().map_err(SwapperError::from)
    }

    pub async fn get_status_by_transaction_hash(&self, transaction_hash: &str) -> Result<Option<BrokerStatusResponse>, SwapperError> {
        let request = StatusByTransactionHashRequest {
            transaction_hash: transaction_hash.to_string(),
        };
        match self.client.get(BrokerTarget::StatusByTransactionHash).query(&request).await {
            Ok(response) => Ok(Some(response)),
            Err(error) if error.status() == Some(404) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn request_deposit_channel(&self, request: &DepositChannelRequest) -> Result<DepositChannelResponse, SwapperError> {
        let response: JsonRpcResult<DepositChannelResponse> = self.deposit_channel_client.post(BrokerTarget::DepositChannel, request).await?;
        response.take().map_err(SwapperError::from)
    }
}
