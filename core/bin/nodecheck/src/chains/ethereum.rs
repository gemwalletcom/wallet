use async_trait::async_trait;
use chain_traits::ChainBalances;
use gem_client::ReqwestClient;
use gem_evm::{
    jsonrpc::TransactionObject,
    rpc::{
        EthereumClient,
        model::{BlockTransactionsIds, Transaction},
    },
};
use num_traits::ToPrimitive;
use serde_json::json;

use crate::{
    checker::{NodeCheck, NodeCheckReporter, NodeCheckResult, check_batch, check_chain, check_expected_rpc_error, method_result},
    fixtures::NodeFixture,
};

const MAX_LOGGED_RESULT_LENGTH: usize = 128;

pub(crate) struct EthereumNodeChecker {
    client: EthereumClient<ReqwestClient>,
}

impl EthereumNodeChecker {
    pub(crate) fn new(client: EthereumClient<ReqwestClient>) -> Self {
        Self { client }
    }

    async fn check_address(&self, address: &str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let balance = ChainBalances::get_balance_coin(&self.client, address.to_string()).await;
        method_result(reporter, "eth_getBalance", balance, |result| result.balance.available.to_string())?;

        let transaction_count = self.client.get_transaction_count(address).await;
        method_result(reporter, "eth_getTransactionCount", transaction_count, |result| result.clone())?;
        Ok(())
    }

    async fn check_transaction(&self, transaction_id: &str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let transaction: Result<Transaction, String> = async {
            let transaction = self
                .client
                .call::<Option<Transaction>>("eth_getTransactionByHash".to_string(), json!([transaction_id]))
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "returned null".to_string())?;
            if !transaction.hash.eq_ignore_ascii_case(transaction_id) {
                return Err(format!("returned {}", transaction.hash));
            }
            Ok(transaction)
        }
        .await;
        let transaction = method_result(reporter, "eth_getTransactionByHash", transaction, |_| "found".to_string())?;

        let receipt: Result<_, String> = async {
            let receipt = self
                .client
                .get_transaction_receipt(transaction_id)
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "returned null".to_string())?;
            if receipt.block_number != transaction.block_number || !receipt.has_valid_block_reference() {
                return Err("invalid block reference".to_string());
            }
            Ok(receipt)
        }
        .await;
        method_result(reporter, "eth_getTransactionReceipt", receipt, |result| format!("block {}", result.block_number))?;

        let block: Result<_, String> = async {
            let block_number = transaction
                .block_number
                .to_u64()
                .ok_or_else(|| format!("transaction block number is too large: {}", transaction.block_number))?;
            let block = self.client.get_block(block_number).await.map_err(|error| error.to_string())?;
            if !block.transactions.iter().any(|transaction| transaction.hash.eq_ignore_ascii_case(transaction_id)) {
                return Err(format!("transaction {transaction_id} is missing"));
            }

            let transaction_ids = self
                .client
                .call::<Option<BlockTransactionsIds>>("eth_getBlockByNumber".to_string(), json!([format!("0x{block_number:x}"), false]))
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "hash-only response returned null".to_string())?;
            if block.timestamp != transaction_ids.timestamp
                || block.transactions.len() != transaction_ids.transactions.len()
                || block
                    .transactions
                    .iter()
                    .zip(&transaction_ids.transactions)
                    .any(|(transaction, transaction_id)| !transaction.hash.eq_ignore_ascii_case(transaction_id))
            {
                return Err("full and hash-only responses do not match".to_string());
            }
            Ok((block_number, block))
        }
        .await;
        let (block_number, block) = method_result(reporter, "eth_getBlockByNumber", block, |(block_number, block)| {
            format!("{block_number}, {} transactions", block.transactions.len())
        })?;

        let receipts: Result<_, String> = async {
            let receipts = self.client.get_block_receipts(block_number).await.map_err(|error| error.to_string())?;
            if receipts.len() != block.transactions.len() || receipts.iter().any(|receipt| receipt.block_number != transaction.block_number) {
                return Err("receipts do not match block transactions".to_string());
            }
            Ok(receipts)
        }
        .await;
        method_result(reporter, "eth_getBlockReceipts", receipts, |result| format!("{} receipts", result.len()))?;
        Ok(())
    }

    async fn check_provider_methods(&self, address: &str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let fee_history = self.client.get_fee_history(1, vec![50]).await.map(|_| ());
        method_result(reporter, "eth_feeHistory", fee_history, |_| "available".to_string())?;

        let call = self.client.eth_call::<String>(address, "0x").await;
        method_result(reporter, "eth_call", call, |result| {
            if result.len() <= MAX_LOGGED_RESULT_LENGTH {
                result.clone()
            } else {
                "available".to_string()
            }
        })?;

        let gas = self.client.estimate_gas(None, address, None, Some("0x")).await;
        method_result(reporter, "eth_estimateGas", gas, |result| result.clone())?;

        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        let trace = self.client.trace_call(&transaction).await.map(|_| ());
        method_result(reporter, "trace_call", trace, |_| "available".to_string())?;

        let call = json!([{ "to": address, "data": "0x" }, "latest"]);
        check_batch(&self.client.client, "eth_call", call, reporter).await
    }
}

#[async_trait]
impl NodeCheck for EthereumNodeChecker {
    async fn check_load_balancer(&self, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        check_chain(&self.client, "eth_chainId", "eth_blockNumber", reporter).await?;
        check_batch(&self.client.client, "eth_chainId", json!([]), reporter).await?;
        check_expected_rpc_error(reporter, "eth_sendRawTransaction", self.client.send_raw_transaction("0x").await)
    }

    async fn check_indexer(&self, fixture: NodeFixture, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let (address, addresses) = fixture.addresses.split_first().ok_or("node fixture has no addresses")?;
        let (transaction_id, transaction_ids) = fixture.transaction_ids.split_first().ok_or("node fixture has no transaction ids")?;

        check_chain(&self.client, "eth_chainId", "eth_blockNumber", reporter).await?;
        self.check_address(address, reporter).await?;
        for address in addresses {
            self.check_address(address, reporter).await?;
        }

        self.check_transaction(transaction_id, reporter).await?;
        for transaction_id in transaction_ids {
            self.check_transaction(transaction_id, reporter).await?;
        }

        self.check_provider_methods(address, reporter).await
    }
}
