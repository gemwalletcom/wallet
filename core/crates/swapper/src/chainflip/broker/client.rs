use super::{
    VaultSwapExtras, VaultSwapResponse,
    model::{ChainflipAsset, DcaParameters},
};
use crate::SwapperError;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use serde_json::{Value, json};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    client: JsonRpcClient<C>,
}

impl<C> BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
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
            VaultSwapExtras::Evm(evm) => serde_json::to_value(evm).unwrap(),
            VaultSwapExtras::Tron(tron) => serde_json::to_value(tron).unwrap(),
            VaultSwapExtras::Solana(sol) => serde_json::to_value(sol).unwrap(),
            VaultSwapExtras::None => Value::Null,
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

        let result = self
            .client
            .call_method_with_param("broker_request_swap_parameter_encoding", params, None)
            .await
            .map_err(SwapperError::from)?;

        result.take().map_err(SwapperError::from)
    }
}
