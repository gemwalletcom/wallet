use crate::{
    jsonrpc::NearRpc,
    models::{Account, AccountAccessKey, Block, BroadcastResult, GasPrice, NodeStatus},
};
use gem_client::Client;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcError};
use primitives::Chain;

#[derive(Debug)]
pub struct NearClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> NearClient<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Near }
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, JsonRpcError> {
        self.client.request(NearRpc::GetAccount(address.to_string())).await
    }

    pub async fn get_account_access_key(&self, address: &str, public_key: &str) -> Result<AccountAccessKey, JsonRpcError> {
        self.client
            .request(NearRpc::GetAccountAccessKey {
                address: address.to_string(),
                public_key: public_key.to_string(),
            })
            .await
    }

    pub async fn get_latest_block(&self) -> Result<Block, JsonRpcError> {
        self.client.request(NearRpc::GetLatestBlock).await
    }

    pub async fn get_gas_price(&self) -> Result<GasPrice, JsonRpcError> {
        self.client.request(NearRpc::GetGasPrice).await
    }

    pub async fn get_status(&self) -> Result<NodeStatus, JsonRpcError> {
        self.client.request(NearRpc::GetStatus).await
    }

    pub async fn broadcast_transaction(&self, signed_transaction: &str) -> Result<BroadcastResult, JsonRpcError> {
        self.client.request(NearRpc::SendTransaction(signed_transaction.to_string())).await
    }

    pub async fn get_transaction(&self, transaction_hash: &str, sender_account_id: &str) -> Result<BroadcastResult, JsonRpcError> {
        self.client
            .request(NearRpc::GetTransactionStatus {
                transaction_hash: transaction_hash.to_string(),
                sender_account_id: sender_account_id.to_string(),
            })
            .await
    }
}
