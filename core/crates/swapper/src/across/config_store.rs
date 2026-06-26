use crate::{
    SwapperError,
    alien::{RpcClient, RpcProvider},
    client_factory::create_client_with_chain,
};
use alloy_primitives::{Address, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_evm::{
    across::{contracts::AcrossConfigStore, fees},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use gem_jsonrpc::{JsonRpcClient, types::JsonRpcResult};
use primitives::{Chain, contract_constants::ETHEREUM_ACROSS_CONFIG_STORE_CONTRACT};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

const CONFIG_CACHE_TTL: u64 = 60 * 60 * 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RateModel {
    #[serde(rename = "UBar")]
    ubar: String,
    #[serde(rename = "R0")]
    r0: String,
    #[serde(rename = "R1")]
    r1: String,
    #[serde(rename = "R2")]
    r2: String,
}

impl From<RateModel> for fees::RateModel {
    fn from(value: RateModel) -> Self {
        Self {
            ubar: value.ubar.parse().unwrap(),
            r0: value.r0.parse().unwrap(),
            r1: value.r1.parse().unwrap(),
            r2: value.r2.parse().unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenConfig {
    pub(super) rate_model: RateModel,
    pub(super) route_rate_model: HashMap<String, RateModel>,
}

pub(super) struct ConfigStoreClient {
    contract: String,
    client: JsonRpcClient<RpcClient>,
}

impl ConfigStoreClient {
    pub(super) fn new(provider: Arc<dyn RpcProvider>) -> ConfigStoreClient {
        ConfigStoreClient {
            contract: ETHEREUM_ACROSS_CONFIG_STORE_CONTRACT.into(),
            client: create_client_with_chain(provider, Chain::Ethereum),
        }
    }

    pub(super) async fn fetch_config(&self, l1token: &Address) -> Result<TokenConfig, SwapperError> {
        let data = AcrossConfigStore::l1TokenConfigCall { l1Token: *l1token }.abi_encode();
        let call = EthereumRpc::Call(TransactionObject::new_call(&self.contract, data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = self.client.call_with_cache(&call, Some(CONFIG_CACHE_TTL)).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(SwapperError::compute_quote_error)?;
        let decoded = AcrossConfigStore::l1TokenConfigCall::abi_decode_returns(&hex_data).map_err(SwapperError::from)?;

        let result: TokenConfig = serde_json::from_str(&decoded).map_err(SwapperError::from)?;
        Ok(result)
    }
}
