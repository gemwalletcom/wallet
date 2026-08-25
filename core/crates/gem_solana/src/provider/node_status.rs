use async_trait::async_trait;
use chain_traits::{
    ChainBalances, ChainToken, ChainTraits,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::{NodeCheckReport, NodeCheckRequest, NodeSyncStatus};
use std::time::Duration;

use crate::{USDC_TOKEN_MINT, method, rpc::SolanaProvider};

const GET_TOKEN_ACCOUNTS_BY_OWNER_MINT_CHECK: &str = "getTokenAccountsByOwner(mint)";
const GET_TOKEN_ACCOUNTS_BY_OWNER_PROGRAM_ID_CHECK: &str = "getTokenAccountsByOwner(programId)";

#[async_trait]
impl<C: Client + Clone> ChainTraits for SolanaProvider<C> {
    async fn check_node(&self, request: &NodeCheckRequest, status: &NodeSyncStatus, status_latency: Duration) -> NodeCheckReport {
        ChainNodeStatus::get_node_status(self, request, status, status_latency).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for SolanaProvider<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, status_latency: Duration, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(
            self,
            status,
            status_latency,
            Some(self.get_chain().network_id()),
            recorder,
            method::GET_GENESIS_HASH,
            method::GET_SLOT,
        )
        .await
    }

    async fn get_node_wallet_status(&self, address: &str, _transaction_id: Option<&str>, _block_number: u64, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let recorder = recorder
            .record_timed(method::GET_BALANCE, async {
                self.get_balance_coin(address.to_string()).await.map(|result| result.balance.available)
            })
            .await;
        let recorder = recorder
            .record_timed(GET_TOKEN_ACCOUNTS_BY_OWNER_PROGRAM_ID_CHECK, async {
                self.get_balance_assets(address.to_string()).await.map(|assets| assets.len())
            })
            .await;
        let recorder = recorder
            .record_timed(GET_TOKEN_ACCOUNTS_BY_OWNER_MINT_CHECK, async {
                self.get_balance_tokens(address.to_string(), vec![USDC_TOKEN_MINT.to_string()])
                    .await
                    .map(|tokens| tokens.len())
            })
            .await;
        let recorder = recorder
            .record_timed(method::GET_LATEST_BLOCKHASH, async {
                self.get_latest_blockhash().await.map(|result| result.value.blockhash)
            })
            .await;
        let recorder = recorder
            .record_timed(method::GET_RECENT_PRIORITIZATION_FEES, async {
                self.get_recent_prioritization_fees().await.map(|result| result.len())
            })
            .await;
        let recorder = recorder
            .record_timed(method::GET_EPOCH_INFO, async { self.get_epoch_info().await.map(|result| result.epoch) })
            .await;
        let recorder = recorder
            .record_timed(method::GET_VOTE_ACCOUNTS, async { self.get_vote_accounts(false).await.map(|result| result.current.len()) })
            .await;
        let recorder = recorder
            .record_timed(method::GET_INFLATION_RATE, async { self.get_inflation_rate().await.map(|result| result.validator) })
            .await;
        let recorder = recorder
            .record_timed(method::GET_SUPPLY, async { self.get_supply().await.map(|result| result.value.total) })
            .await;
        let recorder = recorder
            .record_timed(method::GET_ACCOUNT_INFO, async {
                self.get_token_data(USDC_TOKEN_MINT.to_string()).await.map(|result| result.symbol)
            })
            .await;
        recorder
            .record_timed(method::GET_MULTIPLE_ACCOUNTS, async {
                self.get_multiple_accounts(vec![USDC_TOKEN_MINT.to_string()]).await.map(|result| result.value.len())
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use chain_traits::ChainTraits;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{NodeCheckRequest, NodeCheckStatus, NodeSyncStatus};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::rpc::SolanaClient;

    #[tokio::test]
    async fn test_parser_profile_checks_recent_block() {
        let requested_slots = Arc::new(Mutex::new(Vec::new()));
        let recorded_slots = requested_slots.clone();
        let client = mock_jsonrpc_client(move |method, params| match method {
            method::GET_GENESIS_HASH => Ok(json!("5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d")),
            method::GET_BLOCK => {
                recorded_slots.lock().unwrap().push(params[0].as_u64().unwrap());
                Ok(json!({ "blockTime": 1_700_000_000, "transactions": [] }))
            }
            _ => panic!("unexpected method: {method}"),
        });
        let provider = SolanaProvider::new_rpc_only(SolanaClient::new(client));

        let report = ChainTraits::check_node(&provider, &NodeCheckRequest::Parser, &NodeSyncStatus::synced(1_000), Duration::ZERO).await;

        assert_eq!(*requested_slots.lock().unwrap(), vec![1_000, 990]);
        assert_eq!(report.checks.len(), 4);
        assert_eq!(report.get("block_transactions_latest").unwrap().status, NodeCheckStatus::Passed { result: "0".to_string() });
        assert_eq!(report.get("block_transactions").unwrap().status, NodeCheckStatus::Passed { result: "0".to_string() });
        assert_eq!(
            report.get(method::GET_GENESIS_HASH).unwrap().status,
            NodeCheckStatus::Passed {
                result: "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d".to_string()
            }
        );
        assert_eq!(report.get(method::GET_SLOT).unwrap().status, NodeCheckStatus::Passed { result: "1000".to_string() });
    }
}
