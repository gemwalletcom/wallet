use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::ReqwestClient;
use gem_evm::{
    jsonrpc::TransactionObject,
    rpc::{
        EthereumClient,
        model::{BlockTransactionsIds, Transaction, TransactionReplayTrace},
    },
};
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, ERROR_METHOD_NOT_FOUND, JsonRpcResults};
use gem_tracing::info_with_fields;
use num_traits::ToPrimitive;
use serde_json::{Value, json};

use crate::{checker::NodeCheck, fixtures::NodeFixture};

const MAX_LOG_RESULT_LENGTH: usize = 128;

pub(crate) struct EthereumNodeChecker {
    client: EthereumClient<ReqwestClient>,
}

impl EthereumNodeChecker {
    pub(crate) fn new(client: EthereumClient<ReqwestClient>) -> Self {
        Self { client }
    }

    async fn check_chain(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain_id = ChainState::get_chain_id(&self.client).await.map_err(|error| format!("eth_chainId: {error}"))?;
        let expected_chain_id = self.client.chain.chain_id().to_string();
        if chain_id != expected_chain_id {
            return Err(format!("eth_chainId: expected {expected_chain_id}, received {chain_id}").into());
        }
        info_with_fields!("node check passed", method = "eth_chainId", result = chain_id);

        let block_number = ChainState::get_block_latest_number(&self.client)
            .await
            .map_err(|error| format!("eth_blockNumber: {error}"))?;
        if block_number == 0 {
            return Err("eth_blockNumber: received zero".into());
        }
        info_with_fields!("node check passed", method = "eth_blockNumber", result = block_number);
        Ok(())
    }

    async fn check_batch(&self, calls: Vec<(String, Value)>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let expected_results = calls.len();
        let batch = JsonRpcResults::from(self.client.batch_call::<String>(calls).await.map_err(|error| format!("JSON-RPC batch: {error}"))?)
            .take_all()
            .map_err(|error| format!("JSON-RPC batch: {error}"))?;
        if batch.len() != expected_results {
            return Err(format!("JSON-RPC batch: expected {expected_results} results, received {}", batch.len()).into());
        }
        info_with_fields!("node check passed", method = "json_rpc_batch", result = batch.len());
        Ok(())
    }

    async fn check_send_raw_transaction(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.client.send_raw_transaction("0x").await {
            Ok(hash) => Err(format!("eth_sendRawTransaction: invalid transaction was accepted as {hash}").into()),
            Err(error) => match error.code {
                ERROR_METHOD_NOT_FOUND | ERROR_CLIENT_ERROR => Err(format!("eth_sendRawTransaction: {error}").into()),
                _ => {
                    info_with_fields!("node check passed", method = "eth_sendRawTransaction", error_code = error.code);
                    Ok(())
                }
            },
        }
    }

    async fn check_address(&self, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let balance = self.client.get_eth_balance(address).await.map_err(|error| format!("eth_getBalance({address}): {error}"))?;
        info_with_fields!("node check passed", method = "eth_getBalance", address = address, result = balance);

        let transaction_count = self
            .client
            .get_transaction_count(address)
            .await
            .map_err(|error| format!("eth_getTransactionCount({address}): {error}"))?;
        info_with_fields!("node check passed", method = "eth_getTransactionCount", address = address, result = transaction_count);
        Ok(())
    }

    async fn check_transaction(&self, transaction_id: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let transaction = self
            .client
            .call::<Option<Transaction>>("eth_getTransactionByHash".to_string(), json!([transaction_id]))
            .await
            .map_err(|error| format!("eth_getTransactionByHash({transaction_id}): {error}"))?
            .ok_or_else(|| format!("eth_getTransactionByHash({transaction_id}): returned null"))?;
        if !transaction.hash.eq_ignore_ascii_case(transaction_id) {
            return Err(format!("eth_getTransactionByHash({transaction_id}): returned {}", transaction.hash).into());
        }
        info_with_fields!("node check passed", method = "eth_getTransactionByHash", transaction_id = transaction_id);

        let receipt = self
            .client
            .get_transaction_receipt(transaction_id)
            .await
            .map_err(|error| format!("eth_getTransactionReceipt({transaction_id}): {error}"))?
            .ok_or_else(|| format!("eth_getTransactionReceipt({transaction_id}): returned null"))?;
        if receipt.block_number != transaction.block_number || !receipt.has_valid_block_reference() {
            return Err(format!("eth_getTransactionReceipt({transaction_id}): invalid block reference").into());
        }
        info_with_fields!(
            "node check passed",
            method = "eth_getTransactionReceipt",
            transaction_id = transaction_id,
            block = receipt.block_number
        );

        let block_number = transaction
            .block_number
            .to_u64()
            .ok_or_else(|| format!("transaction block number is too large: {}", transaction.block_number))?;
        let block = self
            .client
            .get_block(block_number)
            .await
            .map_err(|error| format!("eth_getBlockByNumber({block_number}): {error}"))?;
        if !block.transactions.iter().any(|transaction| transaction.hash.eq_ignore_ascii_case(transaction_id)) {
            return Err(format!("eth_getBlockByNumber({block_number}): transaction {transaction_id} is missing").into());
        }

        let block_transaction_ids = self
            .client
            .call::<Option<BlockTransactionsIds>>("eth_getBlockByNumber".to_string(), json!([format!("0x{block_number:x}"), false]))
            .await
            .map_err(|error| format!("eth_getBlockByNumber({block_number}, false): {error}"))?
            .ok_or_else(|| format!("eth_getBlockByNumber({block_number}, false): returned null"))?;
        if block.timestamp != block_transaction_ids.timestamp
            || block.transactions.len() != block_transaction_ids.transactions.len()
            || block
                .transactions
                .iter()
                .zip(&block_transaction_ids.transactions)
                .any(|(transaction, transaction_id)| !transaction.hash.eq_ignore_ascii_case(transaction_id))
        {
            return Err(format!("eth_getBlockByNumber({block_number}): full and hash-only responses do not match").into());
        }
        info_with_fields!(
            "node check passed",
            method = "eth_getBlockByNumber",
            block = block_number,
            transactions = block.transactions.len()
        );

        let receipts = self
            .client
            .get_block_receipts(block_number)
            .await
            .map_err(|error| format!("eth_getBlockReceipts({block_number}): {error}"))?;
        if receipts.len() != block.transactions.len() || receipts.iter().any(|receipt| receipt.block_number != transaction.block_number) {
            return Err(format!("eth_getBlockReceipts({block_number}): receipts do not match block transactions").into());
        }
        info_with_fields!("node check passed", method = "eth_getBlockReceipts", block = block_number, receipts = receipts.len());
        Ok(block_number)
    }

    async fn check_provider_methods(&self, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client.get_fee_history(1, vec![50]).await.map_err(|error| format!("eth_feeHistory: {error}"))?;
        info_with_fields!("node check passed", method = "eth_feeHistory");

        let call_result = self.client.eth_call::<String>(address, "0x").await.map_err(|error| format!("eth_call: {error}"))?;
        if call_result.len() <= MAX_LOG_RESULT_LENGTH {
            info_with_fields!("node check passed", method = "eth_call", result = call_result);
        } else {
            info_with_fields!("node check passed", method = "eth_call");
        }

        let gas_estimate = self
            .client
            .estimate_gas(None, address, None, Some("0x"))
            .await
            .map_err(|error| format!("eth_estimateGas: {error}"))?;
        info_with_fields!("node check passed", method = "eth_estimateGas", result = gas_estimate);

        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        self.client.trace_call(&transaction).await.map_err(|error| format!("trace_call: {error}"))?;
        info_with_fields!("node check passed", method = "trace_call");

        let call = json!([{ "to": address, "data": "0x" }, "latest"]);
        self.check_batch(vec![("eth_call".to_string(), call.clone()), ("eth_call".to_string(), call)]).await
    }

    async fn check_archival_methods(&self, transaction_id: &str, block_number: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.client
            .call::<TransactionReplayTrace>("trace_replayTransaction".to_string(), json!([transaction_id, ["stateDiff"]]))
            .await
            .map_err(|error| format!("trace_replayTransaction({transaction_id}): {error}"))?;
        info_with_fields!("node check passed", method = "trace_replayTransaction", transaction_id = transaction_id);

        self.client
            .trace_replay_block_transactions(block_number)
            .await
            .map_err(|error| format!("trace_replayBlockTransactions({block_number}): {error}"))?;
        info_with_fields!("node check passed", method = "trace_replayBlockTransactions", block = block_number);
        Ok(())
    }
}

#[async_trait]
impl NodeCheck for EthereumNodeChecker {
    async fn check_load_balancer(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.check_chain().await?;
        self.check_batch(vec![("eth_chainId".to_string(), json!([])), ("eth_chainId".to_string(), json!([]))])
            .await?;
        self.check_send_raw_transaction().await
    }

    async fn check_indexer(&self, fixture: NodeFixture, archival: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
        let address = fixture.addresses.first().copied().ok_or("node fixture has no addresses")?;
        let transaction_id = fixture.transaction_ids.first().copied().ok_or("node fixture has no transaction ids")?;

        self.check_chain().await?;
        for address in fixture.addresses {
            self.check_address(address).await?;
        }

        let block_number = self.check_transaction(transaction_id).await?;
        for transaction_id in fixture.transaction_ids.iter().skip(1) {
            self.check_transaction(transaction_id).await?;
        }

        self.check_provider_methods(address).await?;
        if archival {
            self.check_archival_methods(transaction_id, block_number).await?;
        }
        Ok(())
    }
}
