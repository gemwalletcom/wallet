use crate::SwapperError;
use alloy_primitives::{Address, hex::decode as HexDecode};
use alloy_sol_types::SolCall;
use gem_evm::{
    across::{contracts::AcrossConfigStore, fees},
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
};
use primitives::{AssetId, contract_constants::ETHEREUM_ACROSS_CONFIG_STORE_CONTRACT};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RateModel {
    #[serde(rename = "UBar")]
    ubar: String,
    #[serde(rename = "R0")]
    r0: String,
    #[serde(rename = "R1")]
    r1: String,
    #[serde(rename = "R2")]
    r2: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenConfig {
    rate_model: RateModel,
    route_rate_model: HashMap<String, RateModel>,
}

impl TokenConfig {
    pub(super) fn request(l1_token: &Address) -> EthereumRpc {
        let data = AcrossConfigStore::l1TokenConfigCall { l1Token: *l1_token }.abi_encode();
        EthereumRpc::Call(TransactionObject::new_call(ETHEREUM_ACROSS_CONFIG_STORE_CONTRACT, data), BlockParameter::Latest)
    }

    pub(super) fn decode(result: String) -> Result<Self, SwapperError> {
        let data = HexDecode(result).map_err(SwapperError::compute_quote_error)?;
        let decoded = AcrossConfigStore::l1TokenConfigCall::abi_decode_returns(&data).map_err(SwapperError::from)?;
        serde_json::from_str(&decoded).map_err(SwapperError::from)
    }

    pub(super) fn rate_model(&self, from_asset: &AssetId, to_asset: &AssetId) -> Result<fees::RateModel, SwapperError> {
        let route = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let model = self.route_rate_model.get(&route).unwrap_or(&self.rate_model);
        Ok(fees::RateModel {
            ubar: model.ubar.parse().map_err(SwapperError::compute_quote_error)?,
            r0: model.r0.parse().map_err(SwapperError::compute_quote_error)?,
            r1: model.r1.parse().map_err(SwapperError::compute_quote_error)?,
            r2: model.r2.parse().map_err(SwapperError::compute_quote_error)?,
        })
    }
}
