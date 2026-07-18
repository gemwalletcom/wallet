use async_trait::async_trait;
use chain_traits::{ChainBalances, ChainToken, ChainTransactions, TransactionsRequest, TransactionsResult};
use gem_client::ReqwestClient;
use gem_solana::{SolanaClient, USDC_TOKEN_MINT};
use serde_json::{Value, json};

use crate::{
    checker::{NodeCheck, NodeCheckReporter, NodeCheckResult, check_batch, check_chain, check_expected_rpc_error, method_result},
    fixtures::NodeFixture,
};

pub(crate) struct SolanaNodeChecker {
    client: SolanaClient<ReqwestClient>,
}

impl SolanaNodeChecker {
    pub(crate) fn new(client: SolanaClient<ReqwestClient>) -> Self {
        Self { client }
    }

    async fn check_vote_accounts(&self, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let accounts = self.client.get_vote_accounts(false).await;
        method_result(reporter, "getVoteAccounts", accounts, |result| format!("{} validators", result.current.len()))?;
        Ok(())
    }

    async fn check_address(&self, address: &str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let balance = ChainBalances::get_balance_coin(&self.client, address.to_string()).await;
        method_result(reporter, "getBalance", balance, |result| result.balance.available.to_string())?;

        let token_accounts: NodeCheckResult<_> = async {
            let assets = ChainBalances::get_balance_assets(&self.client, address.to_string()).await?;
            let balances = ChainBalances::get_balance_tokens(&self.client, address.to_string(), vec![USDC_TOKEN_MINT.to_string()]).await?;
            Ok((assets, balances))
        }
        .await;
        method_result(reporter, "getTokenAccountsByOwner", token_accounts, |result| {
            format!("{} assets, {} token balances", result.0.len(), result.1.len())
        })?;

        let signatures: NodeCheckResult<_> = async {
            match ChainTransactions::get_transactions_by_address(&self.client, TransactionsRequest::new(address.to_string(), 100)).await? {
                TransactionsResult::TransactionIds(transaction_ids) => Ok(transaction_ids),
                TransactionsResult::Transactions(_) => Err("expected transaction IDs".into()),
            }
        }
        .await;
        method_result(reporter, "getSignaturesForAddress", signatures, |result| format!("{} signatures", result.len()))?;

        let staking_balance = ChainBalances::get_balance_staking(&self.client, address.to_string()).await;
        method_result(reporter, "getProgramAccounts", staking_balance, |result| {
            if result.is_some() { "available".to_string() } else { "none".to_string() }
        })?;
        Ok(())
    }

    async fn check_transaction(&self, transaction_id: &str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let transaction: Result<_, String> = async {
            let state = self
                .client
                .get_transaction(transaction_id)
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "returned null".to_string())?;
            let transaction = ChainTransactions::get_transaction_by_hash(&self.client, transaction_id.to_string())
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "provider returned null".to_string())?;
            if transaction.hash != transaction_id {
                return Err(format!("returned {}", transaction.hash));
            }
            Ok((state.slot, transaction))
        }
        .await;
        let (slot, transaction) = method_result(reporter, "getTransaction", transaction, |(slot, _)| format!("slot {slot}"))?;

        let transactions: Result<_, String> = async {
            let transactions = ChainTransactions::get_transactions_by_block(&self.client, slot).await.map_err(|error| error.to_string())?;
            if !transactions.iter().any(|block_transaction| block_transaction.hash == transaction.hash) {
                return Err(format!("transaction {transaction_id} is missing"));
            }
            Ok(transactions)
        }
        .await;
        method_result(reporter, "getBlock", transactions, |result| format!("{slot}, {} transactions", result.len()))?;
        Ok(())
    }

    async fn check_provider_methods(&self, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let blockhash = self.client.get_latest_blockhash().await;
        method_result(reporter, "getLatestBlockhash", blockhash, |result| result.value.blockhash.clone())?;

        let fees = self.client.get_recent_prioritization_fees().await;
        method_result(reporter, "getRecentPrioritizationFees", fees, |result| format!("{} samples", result.len()))?;

        let epoch = self.client.get_epoch_info().await;
        method_result(reporter, "getEpochInfo", epoch, |result| result.epoch.to_string())?;

        self.check_vote_accounts(reporter).await?;

        let inflation = self.client.get_inflation_rate().await;
        method_result(reporter, "getInflationRate", inflation, |result| result.validator.to_string())?;

        let supply = self.client.get_supply().await;
        method_result(reporter, "getSupply", supply, |result| result.value.total.to_string())?;

        let token = ChainToken::get_token_data(&self.client, USDC_TOKEN_MINT.to_string()).await;
        method_result(reporter, "getAccountInfo", token, |result| result.symbol.clone())?;

        let accounts = self
            .client
            .rpc_call::<Value>("getMultipleAccounts", json!([[USDC_TOKEN_MINT], { "commitment": "confirmed", "encoding": "base64" }]))
            .await;
        method_result(reporter, "getMultipleAccounts", accounts, |_| "available".to_string())?;

        check_expected_rpc_error(reporter, "simulateTransaction", self.client.simulate_encoded_transaction("").await)?;

        check_batch(self.client.get_client(), "getSlot", json!([]), reporter).await
    }
}

#[async_trait]
impl NodeCheck for SolanaNodeChecker {
    async fn check_load_balancer(&self, _fixture: &NodeFixture, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        check_chain(&self.client, "getGenesisHash", "getSlot", reporter).await?;
        self.check_vote_accounts(reporter).await?;
        check_batch(self.client.get_client(), "getSlot", json!([]), reporter).await?;
        check_expected_rpc_error(reporter, "sendTransaction", self.client.send_transaction(String::new(), None).await)
    }

    async fn check_indexer(&self, fixture: &NodeFixture, reporter: &dyn NodeCheckReporter) -> NodeCheckResult {
        let (address, addresses) = fixture.addresses.split_first().ok_or("node fixture has no addresses")?;
        let (transaction_id, transaction_ids) = fixture.transaction_ids.split_first().ok_or("node fixture has no transaction ids")?;

        check_chain(&self.client, "getGenesisHash", "getSlot", reporter).await?;
        self.check_address(address, reporter).await?;
        for address in addresses {
            self.check_address(address, reporter).await?;
        }
        self.check_transaction(transaction_id, reporter).await?;
        for transaction_id in transaction_ids {
            self.check_transaction(transaction_id, reporter).await?;
        }
        self.check_provider_methods(reporter).await
    }
}
