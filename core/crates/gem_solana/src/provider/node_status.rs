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
