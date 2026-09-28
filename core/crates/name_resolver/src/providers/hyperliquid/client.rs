use std::error::Error;

use alloy_ens::namehash;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use gem_jsonrpc::types::JsonRpcError;
use primitives::EVMChain;

use super::contract::{Registrator, Router};
use super::model::Record;

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";
const ERROR_EXECUTION_REVERTED: i32 = 3;

pub struct HyperliquidClient {
    client: EthereumClient<ReqwestClient>,
}

impl HyperliquidClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: EthereumClient::new(JsonRpcClient::new(client), EVMChain::Hyperliquid),
        }
    }

    pub async fn get_record(&self, name: &str) -> Result<Option<Record>, Box<dyn Error + Send + Sync>> {
        let registrator = self.get_registrator().await?;
        let call = Registrator::getFullRecordJSONCall { _namehash: namehash(name) }.abi_encode();
        let result = match self.client.eth_call(&registrator.to_string(), &call).await {
            Ok(result) => result,
            Err(error) if error.downcast_ref::<JsonRpcError>().is_some_and(|error| error.code == ERROR_EXECUTION_REVERTED) => return Ok(None),
            Err(error) => return Err(error),
        };
        let record = Registrator::getFullRecordJSONCall::abi_decode_returns(&result)?;
        Ok(Some(serde_json::from_str(&record)?))
    }

    async fn get_registrator(&self) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let call = Router::getCurrentRegistratorCall {}.abi_encode();
        let result = self.client.eth_call(ROUTER_ADDRESS, &call).await?;
        Ok(Router::getCurrentRegistratorCall::abi_decode_returns(&result)?)
    }
}
