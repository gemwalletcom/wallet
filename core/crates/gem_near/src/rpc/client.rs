use super::{
    NearRpcError,
    mapper::{is_transfer, map_transaction_result},
};
use crate::{
    constants::{BLOCK_REQUEST_CONCURRENCY, EMPTY_TRANSACTION_ROOT},
    jsonrpc::NearRpc,
    models::{Account, AccountAccessKey, AccountAccessKeyList, Block, BroadcastResult, Chunk, GasPrice, NodeStatus, ProtocolConfig},
};
use futures::{StreamExt, TryStreamExt, stream};
use gem_client::Client;
use gem_encoding::encode_base64;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcError};
use primitives::{Chain, Transaction};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::error::Error;

#[derive(Deserialize)]
struct ContractCallResult {
    result: Vec<u8>,
}

#[derive(Debug)]
pub struct NearClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> NearClient<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Near }
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, NearRpcError> {
        self.client.request_with_error(NearRpc::GetAccount { account_id: address.to_string() }).await
    }

    pub async fn account_exists(&self, address: &str) -> Result<bool, NearRpcError> {
        match self.get_account(address).await {
            Ok(_) => Ok(true),
            Err(error) if error.payload.is_unknown_account() => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub async fn call_function<T: Serialize, R: DeserializeOwned>(&self, contract_id: &str, method_name: &str, args: &T) -> Result<R, Box<dyn Error + Sync + Send>> {
        let args = serde_json::to_vec(args)?;
        let response: ContractCallResult = self
            .client
            .request(NearRpc::CallFunction {
                contract_id: contract_id.to_string(),
                method_name: method_name.to_string(),
                args_base64: encode_base64(&args),
            })
            .await?;
        Ok(serde_json::from_slice(&response.result)?)
    }

    pub async fn get_account_access_key(&self, address: &str, public_key: &str) -> Result<AccountAccessKey, JsonRpcError> {
        self.client
            .request(NearRpc::GetAccountAccessKey {
                address: address.to_string(),
                public_key: public_key.to_string(),
            })
            .await
    }

    pub async fn get_account_access_keys(&self, address: &str) -> Result<AccountAccessKeyList, JsonRpcError> {
        self.client.request(NearRpc::GetAccountAccessKeys { address: address.to_string() }).await
    }

    pub async fn get_block(&self, height: u64) -> Result<Block, NearRpcError> {
        self.client.request_with_error(NearRpc::GetBlock { height }).await
    }

    pub async fn get_transactions_by_block(&self, height: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.get_block(height).await?;
        let chunk_requests = block
            .chunks
            .into_iter()
            .filter(|chunk| chunk.height_included == height && chunk.tx_root != EMPTY_TRANSACTION_ROOT)
            .map(|chunk| NearRpc::GetChunk { chunk_hash: chunk.chunk_hash })
            .collect();
        let chunks: Vec<Chunk> = self.request_all(chunk_requests).await?;
        let transaction_requests = chunks
            .into_iter()
            .flat_map(|chunk| chunk.transactions)
            .filter(|transaction| transaction.actions.iter().any(is_transfer))
            .map(|transaction| NearRpc::GetTransactionStatus {
                transaction_hash: transaction.hash,
                sender_account_id: transaction.signer_id,
            })
            .collect();
        let results: Vec<BroadcastResult> = self.request_all(transaction_requests).await?;
        results.into_iter().map(|result| map_transaction_result(result, height, block.header.timestamp)).collect()
    }

    async fn request_all<R: DeserializeOwned + Send>(&self, requests: Vec<NearRpc>) -> Result<Vec<R>, JsonRpcError> {
        stream::iter(requests).map(|request| self.client.request(request)).buffered(BLOCK_REQUEST_CONCURRENCY).try_collect().await
    }

    pub async fn is_skipped_block(&self, height: u64) -> Result<bool, Box<dyn Error + Sync + Send>> {
        match self.get_block(height).await {
            Ok(_) => Ok(false),
            Err(error) if error.payload.is_unknown_block() => Ok(self.get_latest_block().await?.header.height >= height),
            Err(error) => Err(Box::new(error)),
        }
    }

    pub async fn get_latest_block(&self) -> Result<Block, JsonRpcError> {
        self.client.request(NearRpc::GetLatestBlock).await
    }

    pub async fn get_gas_price(&self) -> Result<GasPrice, JsonRpcError> {
        self.client.request(NearRpc::GetGasPrice).await
    }

    pub async fn get_protocol_config(&self) -> Result<ProtocolConfig, JsonRpcError> {
        self.client.request(NearRpc::GetProtocolConfig).await
    }

    pub async fn get_status(&self) -> Result<NodeStatus, JsonRpcError> {
        self.client.request(NearRpc::GetStatus).await
    }

    pub async fn broadcast_transaction(&self, signed_transaction: &str) -> Result<BroadcastResult, JsonRpcError> {
        self.client
            .request(NearRpc::SendTransaction {
                signed_transaction: signed_transaction.to_string(),
            })
            .await
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
