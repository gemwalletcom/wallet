use async_trait::async_trait;
use chain_traits::{
    ChainBalances, ChainToken, ChainTraits,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::{NodeCheckReport, NodeCheckRequest, NodeSyncStatus};

use crate::{USDC_TOKEN_MINT, method, rpc::SolanaProvider};

const GET_TOKEN_ACCOUNTS_BY_OWNER_MINT_CHECK: &str = "getTokenAccountsByOwner(mint)";
const GET_TOKEN_ACCOUNTS_BY_OWNER_PROGRAM_ID_CHECK: &str = "getTokenAccountsByOwner(programId)";

#[async_trait]
impl<C: Client + Clone> ChainTraits for SolanaProvider<C> {
    async fn check_node(&self, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
        ChainNodeStatus::get_node_status(self, request, status).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for SolanaProvider<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(self, status, Some(self.get_chain().network_id()), recorder, method::GET_GENESIS_HASH, method::GET_SLOT).await
    }

    async fn get_node_wallet_status(&self, address: &str, _transaction_id: Option<&str>, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let balance = self.get_balance_coin(address.to_string()).await.map(|result| result.balance.available);
        let recorder = recorder.record(method::GET_BALANCE, balance);

        let assets = self.get_balance_assets(address.to_string()).await.map(|assets| assets.len());
        let recorder = recorder.record(GET_TOKEN_ACCOUNTS_BY_OWNER_PROGRAM_ID_CHECK, assets);

        let tokens = self
            .get_balance_tokens(address.to_string(), vec![USDC_TOKEN_MINT.to_string()])
            .await
            .map(|tokens| tokens.len());
        let recorder = recorder.record(GET_TOKEN_ACCOUNTS_BY_OWNER_MINT_CHECK, tokens);

        let staking_balance = self.get_balance_staking(address.to_string()).await.map(|result| usize::from(result.is_some()));
        let recorder = recorder.record(method::GET_PROGRAM_ACCOUNTS, staking_balance);
        let recorder = recorder.record(method::GET_LATEST_BLOCKHASH, self.get_latest_blockhash().await.map(|result| result.value.blockhash));
        let recorder = recorder.record(
            method::GET_RECENT_PRIORITIZATION_FEES,
            self.get_recent_prioritization_fees().await.map(|result| result.len()),
        );
        let recorder = recorder.record(method::GET_EPOCH_INFO, self.get_epoch_info().await.map(|result| result.epoch));
        let recorder = recorder.record(method::GET_VOTE_ACCOUNTS, self.get_vote_accounts(false).await.map(|result| result.current.len()));
        let recorder = recorder.record(method::GET_INFLATION_RATE, self.get_inflation_rate().await.map(|result| result.validator));
        let recorder = recorder.record(method::GET_SUPPLY, self.get_supply().await.map(|result| result.value.total));
        let recorder = recorder.record(method::GET_ACCOUNT_INFO, self.get_token_data(USDC_TOKEN_MINT.to_string()).await.map(|result| result.symbol));
        recorder.record(
            method::GET_MULTIPLE_ACCOUNTS,
            self.get_multiple_accounts(vec![USDC_TOKEN_MINT.to_string()]).await.map(|result| result.value.len()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chain_traits::ChainTraits;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{NodeCheckReport, NodeCheckRequest, NodeCheckStatus, NodeSyncStatus};
    use serde_json::json;

    use super::*;
    use crate::rpc::SolanaClient;

    #[tokio::test]
    async fn test_parser_profile_checks_recent_block() {
        let client = mock_jsonrpc_client(|method, params| match method {
            method::GET_GENESIS_HASH => Ok(json!("5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d")),
            method::GET_BLOCK => {
                assert_eq!(params[0], 990);
                Ok(json!({ "blockTime": 1_700_000_000, "transactions": [] }))
            }
            _ => panic!("unexpected method: {method}"),
        });
        let provider = SolanaProvider::new_rpc_only(SolanaClient::new(client));

        let report = ChainTraits::check_node(&provider, &NodeCheckRequest::Parser, &NodeSyncStatus::synced(1_000)).await;

        assert_eq!(
            report,
            NodeCheckReport {
                checks: BTreeMap::from([
                    ("block_transactions".to_string(), NodeCheckStatus::Passed { result: "0".to_string() }),
                    (
                        method::GET_GENESIS_HASH.to_string(),
                        NodeCheckStatus::Passed {
                            result: "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d".to_string()
                        }
                    ),
                    (method::GET_SLOT.to_string(), NodeCheckStatus::Passed { result: "1000".to_string() }),
                ])
            }
        );
    }
}
