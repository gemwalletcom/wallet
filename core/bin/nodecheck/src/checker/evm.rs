use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_evm::{
    jsonrpc::TransactionObject,
    rpc::{
        EthereumClient,
        model::{BlockTransactionsIds, Transaction, TransactionReplayTrace},
    },
};
use gem_jsonrpc::client::JsonRpcClient;
use gem_tracing::info_with_fields;
use num_traits::ToPrimitive;
use primitives::EVMChain;
use serde_json::{Value, json};

use crate::fixtures::NodeFixture;

use super::{NodeCheck, NodeCheckResult};

pub(crate) struct EvmChecker {
    chain: EVMChain,
    client: EthereumClient<ReqwestClient>,
    fixture: NodeFixture,
    archival: bool,
}

impl EvmChecker {
    pub(crate) fn new(chain: EVMChain, url: String, fixture: NodeFixture, archival: bool) -> Self {
        Self {
            chain,
            client: EthereumClient::new(JsonRpcClient::new_reqwest(url), chain),
            fixture,
            archival,
        }
    }

    async fn check_methods(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let address = self.fixture.addresses.first().ok_or("node fixture has no addresses")?;
        let transaction_hash = self.fixture.transaction_hashes.first().ok_or("node fixture has no transaction hashes")?;
        self.check_chain().await?;
        for address in self.fixture.addresses {
            self.check_address(address).await?;
        }
        for hash in self.fixture.transaction_hashes {
            self.check_transaction(hash).await?;
        }
        self.check_common_methods(address).await?;
        if self.archival {
            self.check_archival_methods(transaction_hash).await?;
        }
        Ok(())
    }

    async fn check_chain(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain_id = self.client.get_chain_id().await.map_err(|error| format!("eth_chainId: {error}"))?;
        let chain_id = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;
        if chain_id != self.chain.chain_id() {
            return Err(format!("eth_chainId: expected {}, received {chain_id}", self.chain.chain_id()).into());
        }
        info_with_fields!("node check passed", method = "eth_chainId");

        let block_number = self.client.get_latest_block().await.map_err(|error| format!("eth_blockNumber: {error}"))?;
        if block_number == 0 {
            return Err("eth_blockNumber: received zero".into());
        }
        info_with_fields!("node check passed", method = "eth_blockNumber");
        Ok(())
    }

    async fn check_address(&self, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client.get_eth_balance(address).await.map_err(|error| format!("eth_getBalance({address}): {error}"))?;
        info_with_fields!("node check passed", method = "eth_getBalance", fixture = address);
        self.client
            .get_transaction_count(address)
            .await
            .map_err(|error| format!("eth_getTransactionCount({address}): {error}"))?;
        info_with_fields!("node check passed", method = "eth_getTransactionCount", fixture = address);
        self.client.get_code(address).await.map_err(|error| format!("eth_getCode({address}): {error}"))?;
        info_with_fields!("node check passed", method = "eth_getCode", fixture = address);
        Ok(())
    }

    async fn check_transaction(&self, hash: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let transaction = self
            .client
            .call::<Option<Transaction>>("eth_getTransactionByHash".to_string(), json!([hash]))
            .await
            .map_err(|error| format!("eth_getTransactionByHash({hash}): {error}"))?
            .ok_or_else(|| format!("eth_getTransactionByHash({hash}): returned null"))?;
        if !transaction.hash.eq_ignore_ascii_case(hash) {
            return Err(format!("eth_getTransactionByHash({hash}): returned {}", transaction.hash).into());
        }
        info_with_fields!("node check passed", method = "eth_getTransactionByHash", fixture = hash);

        let receipt = self
            .client
            .get_transaction_receipt(hash)
            .await
            .map_err(|error| format!("eth_getTransactionReceipt({hash}): {error}"))?
            .ok_or_else(|| format!("eth_getTransactionReceipt({hash}): returned null"))?;
        if receipt.block_number != transaction.block_number || !receipt.has_valid_block_reference() {
            return Err(format!("eth_getTransactionReceipt({hash}): invalid block reference").into());
        }
        info_with_fields!("node check passed", method = "eth_getTransactionReceipt", fixture = hash);

        let block_number = transaction
            .block_number
            .to_u64()
            .ok_or_else(|| format!("transaction block number is too large: {}", transaction.block_number))?;
        let block = self
            .client
            .get_block(block_number)
            .await
            .map_err(|error| format!("eth_getBlockByNumber({block_number}): {error}"))?;
        if !block.transactions.iter().any(|transaction| transaction.hash.eq_ignore_ascii_case(hash)) {
            return Err(format!("eth_getBlockByNumber({block_number}): transaction {hash} is missing").into());
        }
        info_with_fields!("node check passed", method = "eth_getBlockByNumber", block = block_number, full = true);

        let block_ids = self
            .client
            .call::<Option<BlockTransactionsIds>>("eth_getBlockByNumber".to_string(), json!([format!("0x{block_number:x}"), false]))
            .await
            .map_err(|error| format!("eth_getBlockByNumber({block_number}, false): {error}"))?
            .ok_or_else(|| format!("eth_getBlockByNumber({block_number}, false): returned null"))?;
        if !block_ids.transactions.iter().any(|transaction_hash| transaction_hash.eq_ignore_ascii_case(hash)) {
            return Err(format!("eth_getBlockByNumber({block_number}, false): transaction {hash} is missing").into());
        }
        info_with_fields!("node check passed", method = "eth_getBlockByNumber", block = block_number, full = false);

        let receipts = self
            .client
            .get_block_receipts(block_number)
            .await
            .map_err(|error| format!("eth_getBlockReceipts({block_number}): {error}"))?;
        if receipts.len() != block.transactions.len() || receipts.iter().any(|receipt| receipt.block_number != transaction.block_number) {
            return Err(format!("eth_getBlockReceipts({block_number}): receipts do not match block transactions").into());
        }
        info_with_fields!("node check passed", method = "eth_getBlockReceipts", block = block_number);
        Ok(())
    }

    async fn check_common_methods(&self, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client.gas_price().await.map_err(|error| format!("eth_gasPrice: {error}"))?;
        info_with_fields!("node check passed", method = "eth_gasPrice");
        self.client.get_fee_history(1, vec![50]).await.map_err(|error| format!("eth_feeHistory: {error}"))?;
        info_with_fields!("node check passed", method = "eth_feeHistory");

        self.client
            .call::<String>("eth_call".to_string(), json!([{ "to": address, "data": "0x" }, "latest"]))
            .await
            .map_err(|error| format!("eth_call: {error}"))?;
        info_with_fields!("node check passed", method = "eth_call");
        self.client
            .estimate_gas(None, address, None, Some("0x"))
            .await
            .map_err(|error| format!("eth_estimateGas: {error}"))?;
        info_with_fields!("node check passed", method = "eth_estimateGas");
        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        self.client.trace_call(&transaction).await.map_err(|error| format!("trace_call: {error}"))?;
        info_with_fields!("node check passed", method = "trace_call");

        let syncing = self
            .client
            .call::<Value>("eth_syncing".to_string(), json!([]))
            .await
            .map_err(|error| format!("eth_syncing: {error}"))?;
        if !syncing.is_boolean() && !syncing.is_object() {
            return Err(format!("eth_syncing: invalid response {syncing}").into());
        }
        info_with_fields!("node check passed", method = "eth_syncing");

        let batch = self
            .client
            .client
            .batch_call::<String>(vec![("eth_chainId".to_string(), json!([])), ("eth_blockNumber".to_string(), json!([]))])
            .await
            .map_err(|error| format!("JSON-RPC batch: {error}"))?
            .take_all()
            .map_err(|error| format!("JSON-RPC batch: {error}"))?;
        if batch.len() != 2 {
            return Err(format!("JSON-RPC batch: expected 2 results, received {}", batch.len()).into());
        }
        info_with_fields!("node check passed", method = "json_rpc_batch");

        match self.client.send_raw_transaction("0x").await {
            Ok(hash) => Err(format!("eth_sendRawTransaction: invalid transaction was accepted as {hash}").into()),
            Err(error) if error.code == -32601 => Err(format!("eth_sendRawTransaction: {error}").into()),
            Err(_) => {
                info_with_fields!("node check passed", method = "eth_sendRawTransaction");
                Ok(())
            }
        }
    }

    async fn check_archival_methods(&self, hash: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client
            .call::<TransactionReplayTrace>("trace_replayTransaction".to_string(), json!([hash, ["stateDiff"]]))
            .await
            .map_err(|error| format!("trace_replayTransaction({hash}): {error}"))?;
        let receipt = self
            .client
            .get_transaction_receipt(hash)
            .await?
            .ok_or_else(|| format!("eth_getTransactionReceipt({hash}): returned null"))?;
        let block_number = receipt.block_number.to_u64().ok_or("receipt block number is too large")?;
        self.client
            .trace_replay_block_transactions(block_number)
            .await
            .map_err(|error| format!("trace_replayBlockTransactions({block_number}): {error}"))?;
        info_with_fields!("node check passed", method = "trace_replayTransaction", fixture = hash);
        info_with_fields!("node check passed", method = "trace_replayBlockTransactions", block = block_number);
        Ok(())
    }
}

#[async_trait]
impl NodeCheck for EvmChecker {
    async fn check(&self) -> Result<NodeCheckResult, Box<dyn Error + Send + Sync>> {
        self.check_methods().await?;
        Ok(NodeCheckResult::Evm {
            chain: self.chain,
            addresses: self.fixture.addresses.len(),
            transactions: self.fixture.transaction_hashes.len(),
            archival: self.archival,
        })
    }
}
