use super::{
    AssetsResponse, QuoteRequest, QuoteResponse, VaultSwapExtras, VaultSwapResponse,
    jsonrpc::RequestSwapParameterEncoding,
    model::{ChainflipAsset, DcaParameters},
};
use crate::SwapperError;
use gem_client::{Client, ClientExt, build_path_with_query};
use gem_jsonrpc::types::{JsonRpcResult, ToJsonRpcRequest};
use serde_json::{Value, json};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_assets(&self) -> Result<AssetsResponse, SwapperError> {
        self.client.get("/assets").await.map_err(SwapperError::from)
    }

    pub async fn get_quotes(&self, request: &QuoteRequest) -> Result<Vec<QuoteResponse>, SwapperError> {
        let path = build_path_with_query("/quotes-native", request)?;
        self.client.get(&path).await.map_err(SwapperError::from)
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
        let extra_params_json = match extra_params {
            VaultSwapExtras::Evm(evm) => serde_json::to_value(evm)?,
            VaultSwapExtras::Tron(tron) => serde_json::to_value(tron)?,
            VaultSwapExtras::Solana(sol) => serde_json::to_value(sol)?,
        };

        let params = json!([
            source_asset,
            destination_asset,
            destination_address,
            broker_commission,
            extra_params_json,
            Value::Null,
            boost_fee,
            Vec::<Value>::new(),
            dca_params,
        ]);

        let request = RequestSwapParameterEncoding(params).to_jsonrpc_request(1);
        let response: JsonRpcResult<VaultSwapResponse> = self.client.post("/rpc", &request).await?;
        response.take().map_err(SwapperError::from)
    }
}
