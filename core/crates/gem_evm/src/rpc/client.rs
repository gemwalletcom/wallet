use alloy_primitives::{Address, hex};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::types::{ERROR_INTERNAL_ERROR, JsonRpcError};

use num_bigint::{BigInt, BigUint, Sign};
use serde_json::json;
use serde_serializers::biguint_from_hex_str;
use std::str::FromStr;

use super::model::{Block, BlockHeader, TraceCallResult, TransactionReceipt};
use crate::jsonrpc::{BlockParameter, EthereumRpc, TransactionObject};
use crate::models::fee::EthereumFeeHistory;
#[cfg(feature = "rpc")]
use crate::multicall3::{
    IMulticall3,
    IMulticall3::{Call3, Result as MulticallResult},
    deployment_by_chain,
};
#[cfg(feature = "rpc")]
use alloy_sol_types::SolCall;
use primitives::{Chain, EVMChain};

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

#[derive(Debug, Clone)]
pub struct EthereumClient<C: Client + Clone> {
    pub(crate) chain: EVMChain,
    pub(crate) client: JsonRpcClient<C>,
}

impl<C: Client + Clone> EthereumClient<C> {
    pub fn new(client: JsonRpcClient<C>, chain: EVMChain) -> Self {
        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.to_chain()
    }

    pub async fn eth_call(&self, contract_address: &str, call_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let to_address = Address::from_str(contract_address)?;
        let transaction = TransactionObject::new_call(&to_address.to_string(), call_data.to_vec());
        let result: String = self.client.request(EthereumRpc::Call(transaction, BlockParameter::Latest)).await?;
        Ok(hex::decode(result)?)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<Option<Block>, JsonRpcError> {
        self.client.request(EthereumRpc::GetBlockByNumber(block_number, true)).await
    }

    pub async fn get_block_timestamp(&self, block_number: u64) -> Result<BigUint, JsonRpcError> {
        Ok(self
            .client
            .request::<EthereumRpc, BlockHeader>(EthereumRpc::GetBlockByNumber(block_number, false))
            .await?
            .timestamp)
    }

    pub async fn get_block_receipts(&self, block_number: u64) -> Result<Vec<TransactionReceipt>, JsonRpcError> {
        self.client.request(EthereumRpc::GetBlockReceipts(block_number)).await
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let block_hex: String = self.client.request(EthereumRpc::BlockNumber).await?;
        let block_hex = block_hex.trim_start_matches("0x");
        Ok(u64::from_str_radix(block_hex, 16)?)
    }

    pub async fn get_transaction_receipt(&self, hash: &str) -> Result<Option<TransactionReceipt>, JsonRpcError> {
        self.client.request(EthereumRpc::GetTransactionReceipt(hash.to_string())).await
    }

    pub async fn trace_call(&self, transaction: &TransactionObject) -> Result<TraceCallResult, JsonRpcError> {
        self.client.request(EthereumRpc::TraceCall(transaction.clone(), BlockParameter::Latest)).await
    }

    pub async fn get_balance(&self, address: &str) -> Result<String, JsonRpcError> {
        self.client.request(EthereumRpc::GetBalance(address.to_string(), BlockParameter::Latest)).await
    }

    pub async fn get_code(&self, address: &str) -> Result<String, JsonRpcError> {
        self.client.request(EthereumRpc::GetCode(address.to_string(), BlockParameter::Latest)).await
    }

    pub async fn get_gas_price(&self) -> Result<BigInt, JsonRpcError> {
        let value: String = self.client.request(EthereumRpc::GasPrice).await?;
        let biguint = biguint_from_hex_str(&value).map_err(|_| JsonRpcError {
            code: ERROR_INTERNAL_ERROR,
            message: format!("Failed to parse gas price: {value}"),
            cause: None,
        })?;
        Ok(BigInt::from_biguint(Sign::Plus, biguint))
    }

    pub async fn get_chain_id(&self) -> Result<String, JsonRpcError> {
        self.client.request(EthereumRpc::ChainId).await
    }

    pub async fn get_transaction_count(&self, address: &str) -> Result<String, JsonRpcError> {
        self.client.request(EthereumRpc::GetTransactionCount(address.to_string(), BlockParameter::Latest)).await
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<String, JsonRpcError> {
        self.client.request(EthereumRpc::SendRawTransaction(data.to_string())).await
    }

    pub async fn batch_eth_call<const N: usize>(&self, contract_address: &str, function_selectors: [&str; N]) -> Result<[String; N], Box<dyn std::error::Error + Sync + Send>> {
        let requests: Vec<EthereumRpc> = function_selectors
            .iter()
            .map(|selector| hex::decode(selector).map(|data| EthereumRpc::Call(TransactionObject::new_call(contract_address, data), BlockParameter::Latest)))
            .collect::<Result<_, _>>()?;
        let results = self.client.batch_request::<EthereumRpc, String>(requests).await?.take_all()?;
        results.try_into().map_err(|_| "Array conversion failed".into())
    }

    pub async fn get_fee_history(&self, blocks: u64, reward_percentiles: Vec<u64>) -> Result<EthereumFeeHistory, JsonRpcError> {
        self.client.request(EthereumRpc::FeeHistory { blocks, reward_percentiles }).await
    }

    pub async fn batch_token_balance_calls(&self, address: &str, contracts: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let data = hex::decode(format!("0x70a08231000000000000000000000000{:0>40}", address.strip_prefix("0x").unwrap_or(address)))?;
        let requests: Vec<EthereumRpc> = contracts
            .iter()
            .map(|contract| EthereumRpc::Call(TransactionObject::new_call(contract, data.clone()), BlockParameter::Latest))
            .collect();
        Ok(self.client.batch_request(requests).await?.take_all()?)
    }

    pub async fn estimate_gas(&self, from: Option<&str>, to: &str, value: Option<&str>, data: Option<&str>) -> Result<String, JsonRpcError> {
        let mut params_obj = json!({
            "to": to
        });

        if let Some(from) = from {
            params_obj["from"] = json!(from);
        }

        if let Some(value) = value {
            params_obj["value"] = json!(value);
        }
        if let Some(data) = data {
            params_obj["data"] = json!(data);
        }

        self.client.request(EthereumRpc::EstimateGas(params_obj, BlockParameter::Latest)).await
    }

    #[cfg(feature = "rpc")]
    pub async fn multicall3(&self, calls: Vec<Call3>) -> Result<Vec<MulticallResult>, Box<dyn std::error::Error + Sync + Send>> {
        let target = Address::from_str(deployment_by_chain(&self.chain))?;
        self.call_contract(target, IMulticall3::aggregate3Call { calls }).await
    }

    #[cfg(feature = "rpc")]
    pub async fn call_contract<T: SolCall>(&self, target: Address, sol_call: T) -> Result<T::Return, Box<dyn std::error::Error + Sync + Send>> {
        Ok(T::abi_decode_returns(&self.eth_call(&target.to_string(), &sol_call.abi_encode()).await?)?)
    }

    #[cfg(feature = "rpc")]
    pub async fn multicall3_map<T, R, const N: usize>(
        &self,
        items: &[T],
        build: impl Fn(&T) -> [Call3; N],
        decode: impl Fn(&[MulticallResult]) -> Result<R, Box<dyn std::error::Error + Send + Sync>>,
    ) -> Result<Vec<R>, Box<dyn std::error::Error + Sync + Send>> {
        let calls = items.iter().flat_map(&build).collect();
        let results = self.multicall3(calls).await?;
        results.chunks(N).map(&decode).collect()
    }
}
