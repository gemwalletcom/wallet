use async_trait::async_trait;
use chain_traits::{
    ChainBalances, ChainToken,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::{SolanaClient, USDC_TOKEN_MINT, method, models::SingleTransaction};

const GET_TOKEN_ACCOUNTS_BY_OWNER_MINT_CHECK: &str = "getTokenAccountsByOwner(mint)";
const GET_TOKEN_ACCOUNTS_BY_OWNER_PROGRAM_ID_CHECK: &str = "getTokenAccountsByOwner(programId)";

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for SolanaClient<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(self, status, Some(self.get_chain().network_id()), recorder, method::GET_GENESIS_HASH, method::GET_SLOT).await
    }

    async fn get_node_wallet_status(&self, address: &str, _transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
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

    async fn get_node_parser_status(&self, address: &str, transaction_id: &str, _status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let signatures = self.get_signatures_for_address(address, 100).await.map(|signatures| signatures.len());
        let recorder = recorder.record(method::GET_SIGNATURES_FOR_ADDRESS, signatures);

        let transaction = self
            .get_transaction::<SingleTransaction>(transaction_id)
            .await
            .map_err(|error| error.to_string())
            .and_then(|transaction| transaction.ok_or_else(|| "returned null".to_string()))
            .map(|transaction| transaction.slot);
        let (recorder, slot) = recorder.record_value(method::GET_TRANSACTION, transaction);
        let Some(slot) = slot else { return recorder };

        recorder.record_available(method::GET_BLOCK, self.get_block_transactions(slot).await)
    }
}
