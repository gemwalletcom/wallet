use std::error::Error;

use alloy_ens::namehash;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use primitives::EVMChain;

use super::contract::{Registrator, Router};
use super::model::Record;

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";

pub struct HyperliquidClient {
    client: EthereumClient<ReqwestClient>,
}

impl HyperliquidClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: EthereumClient::new(JsonRpcClient::new(client), EVMChain::Hyperliquid),
        }
    }

    pub async fn get_record(&self, name: &str) -> Result<Record, Box<dyn Error + Send + Sync>> {
        let registrator = self.get_registrator().await?;
        let call = Registrator::getFullRecordJSONCall { _namehash: namehash(name) }.abi_encode();
        let result = self.client.eth_call(&registrator.to_string(), &call).await?;
        let record = Registrator::getFullRecordJSONCall::abi_decode_returns(&result)?;
        Ok(serde_json::from_str(&record)?)
    }

    async fn get_registrator(&self) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let call = Router::getCurrentRegistratorCall {}.abi_encode();
        let result = self.client.eth_call(ROUTER_ADDRESS, &call).await?;
        Ok(Router::getCurrentRegistratorCall::abi_decode_returns(&result)?)
    }
}
