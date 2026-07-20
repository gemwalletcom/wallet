use std::error::Error;

use chain_traits::{ChainBalances, ChainState, ChainToken, ChainTransactions, NodeCheckRecorder, NodeCheckReporter, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, ERROR_METHOD_NOT_FOUND, JsonRpcError};
use primitives::{NodeCheckProfile, NodeCheckReport};
use serde_json::{Value, json};

use crate::{SolanaClient, USDC_TOKEN_MINT};

const ADDRESS: &str = "37BenMAXFJMo3GaXKb2XLsNQXmd6VbbdShZWnwDj9D6k";
const TRANSACTION_ID: &str = "4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx";
const TRANSACTION_SLOT: u64 = 355393521;

impl<C: Client + Clone> SolanaClient<C> {
    pub(super) async fn check_node_profile(&self, profile: NodeCheckProfile, reporter: &dyn NodeCheckReporter) -> NodeCheckReport {
        let mut recorder = NodeCheckRecorder::new(reporter);
        if !self.check_chain(&mut recorder).await {
            return recorder.finish();
        }

        match profile {
            NodeCheckProfile::LoadBalancer => self.check_load_balancer(&mut recorder).await,
            NodeCheckProfile::Parser | NodeCheckProfile::ArchivalParser => self.check_parser(&mut recorder).await,
        }
        recorder.finish()
    }

    async fn check_chain(&self, recorder: &mut NodeCheckRecorder<'_>) -> bool {
        let chain = self.get_chain();
        let expected = chain.network_id();
        let chain_id = ChainState::get_chain_id(self).await.map_err(|error| error.to_string()).and_then(|chain_id| {
            if chain_id == expected {
                Ok(chain_id)
            } else {
                Err(format!("expected {expected}, received {chain_id}"))
            }
        });
        if recorder.record("getGenesisHash", chain_id, Clone::clone).is_none() {
            return false;
        }

        let slot = ChainState::get_block_latest_number(self)
            .await
            .map_err(|error| error.to_string())
            .and_then(|slot| if slot > 0 { Ok(slot) } else { Err("received zero".to_string()) });
        recorder.record("getSlot", slot, ToString::to_string).is_some()
    }

    async fn check_load_balancer(&self, recorder: &mut NodeCheckRecorder<'_>) {
        let balance = ChainBalances::get_balance_coin(self, ADDRESS.to_string()).await;
        recorder.record("getBalance", balance, |result| result.balance.available.to_string());

        let token_accounts = async {
            let assets = ChainBalances::get_balance_assets(self, ADDRESS.to_string()).await?;
            ChainBalances::get_balance_tokens(self, ADDRESS.to_string(), vec![USDC_TOKEN_MINT.to_string()]).await?;
            Ok::<_, Box<dyn Error + Send + Sync>>(assets.len())
        }
        .await;
        recorder.record("getTokenAccountsByOwner", token_accounts, ToString::to_string);

        let staking_balance = ChainBalances::get_balance_staking(self, ADDRESS.to_string()).await;
        recorder.record("getProgramAccounts", staking_balance, |result| usize::from(result.is_some()).to_string());

        let blockhash = self.get_latest_blockhash().await;
        recorder.record("getLatestBlockhash", blockhash, |result| result.value.blockhash.clone());

        let fees = self.get_recent_prioritization_fees().await;
        recorder.record("getRecentPrioritizationFees", fees, |result| result.len().to_string());

        let epoch = self.get_epoch_info().await;
        recorder.record("getEpochInfo", epoch, |result| result.epoch.to_string());

        let vote_accounts = self.get_vote_accounts(false).await;
        recorder.record("getVoteAccounts", vote_accounts, |result| result.current.len().to_string());

        let inflation = self.get_inflation_rate().await;
        recorder.record("getInflationRate", inflation, |result| result.validator.to_string());

        let supply = self.get_supply().await;
        recorder.record("getSupply", supply, |result| result.value.total.to_string());

        let token = ChainToken::get_token_data(self, USDC_TOKEN_MINT.to_string()).await;
        recorder.record("getAccountInfo", token, |result| result.symbol.clone());

        let accounts = self
            .rpc_call::<Value>("getMultipleAccounts", json!([[USDC_TOKEN_MINT], { "commitment": "confirmed", "encoding": "base64" }]))
            .await;
        recorder.record("getMultipleAccounts", accounts, |_| "available".to_string());

        let simulation = expected_rpc_error(self.simulate_encoded_transaction("").await);
        recorder.record("simulateTransaction", simulation, ToString::to_string);

        let calls = vec![("getSlot".to_string(), json!([])), ("getSlot".to_string(), json!([]))];
        let batch = self.get_client().batch_call::<Value>(calls).await.and_then(|results| results.take_all());
        recorder.record("json_rpc_batch", batch, |results| results.len().to_string());

        let broadcast = expected_rpc_error(self.send_transaction(String::new(), None).await);
        recorder.record("sendTransaction", broadcast, ToString::to_string);
    }

    async fn check_parser(&self, recorder: &mut NodeCheckRecorder<'_>) {
        let signatures: Result<usize, Box<dyn Error + Send + Sync>> = async {
            match ChainTransactions::get_transactions_by_address(self, TransactionsRequest::new(ADDRESS.to_string(), 100)).await? {
                TransactionsResult::TransactionIds(transaction_ids) => Ok(transaction_ids.len()),
                TransactionsResult::Transactions(_) => Err("expected transaction IDs".into()),
            }
        }
        .await;
        recorder.record("getSignaturesForAddress", signatures, ToString::to_string);

        let transaction = ChainTransactions::get_transaction_by_hash(self, TRANSACTION_ID.to_string())
            .await
            .map_err(|error| error.to_string())
            .and_then(|transaction| transaction.ok_or_else(|| "returned null".to_string()))
            .and_then(|transaction| {
                if transaction.hash == TRANSACTION_ID {
                    Ok(transaction)
                } else {
                    Err(format!("returned {}", transaction.hash))
                }
            });
        let Some(transaction) = recorder.record("getTransaction", transaction, |_| TRANSACTION_SLOT.to_string()) else {
            return;
        };

        let transactions = ChainTransactions::get_transactions_by_block(self, TRANSACTION_SLOT)
            .await
            .map_err(|error| error.to_string())
            .and_then(|transactions| {
                if transactions.iter().any(|block_transaction| block_transaction.hash == transaction.hash) {
                    Ok(transactions)
                } else {
                    Err(format!("transaction {TRANSACTION_ID} is missing"))
                }
            });
        recorder.record("getBlock", transactions, |_| TRANSACTION_SLOT.to_string());
    }
}

fn expected_rpc_error<T>(result: Result<T, JsonRpcError>) -> Result<i32, String> {
    match result {
        Ok(_) => Err("invalid request was accepted".to_string()),
        Err(error) => match error.code {
            ERROR_METHOD_NOT_FOUND | ERROR_CLIENT_ERROR => Err(error.to_string()),
            _ => Ok(error.code),
        },
    }
}
