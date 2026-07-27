mod receipt_history;

use std::time::Duration;

use async_trait::async_trait;
use chain_traits::{
    ChainBalances, ChainTraits,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::{Chain, NodeCheckReport, NodeCheckRequest, NodeSyncStatus};

use crate::{jsonrpc::TransactionObject, method, rpc::EthereumProvider};
use receipt_history::record_receipt_checks;

const ETH_CALL_MONAD_DELEGATIONS_CHECK: &str = "eth_call_monad_delegations";

#[async_trait]
impl<C: Client + Clone> ChainTraits for EthereumProvider<C> {
    async fn check_node(&self, request: &NodeCheckRequest, status: &NodeSyncStatus, status_latency: Duration) -> NodeCheckReport {
        ChainNodeStatus::get_node_status(self, request, status, status_latency).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for EthereumProvider<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, status_latency: Duration, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(
            self,
            status,
            status_latency,
            Some(self.get_chain().network_id()),
            recorder,
            method::ETH_CHAIN_ID,
            method::ETH_BLOCK_NUMBER,
        )
        .await
    }

    async fn get_node_wallet_status(&self, address: &str, _transaction_id: Option<&str>, block_number: u64, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let recorder = recorder
            .record_timed(method::ETH_GET_BALANCE, async {
                self.get_balance_coin(address.to_string()).await.map(|result| result.balance.available)
            })
            .await;
        let recorder = recorder.record_timed(method::ETH_GET_TRANSACTION_COUNT, self.get_transaction_count(address)).await;
        let recorder = record_receipt_checks(self, recorder, block_number).await;
        let recorder = recorder.record_available_timed(method::ETH_FEE_HISTORY, self.get_fee_history(1, vec![50])).await;
        let recorder = recorder.record_timed(method::ETH_GAS_PRICE, self.get_gas_price()).await;
        let recorder = recorder.record_timed(method::ETH_GET_CODE, self.get_code(address)).await;
        let recorder = recorder.record_available_timed(method::ETH_CALL, self.eth_call(address, &[])).await;
        let recorder = if self.get_chain() == Chain::Monad {
            recorder
                .record_available_timed(ETH_CALL_MONAD_DELEGATIONS_CHECK, self.call_monad_delegations(address))
                .await
        } else {
            recorder
        };
        let recorder = recorder.record_timed(method::ETH_ESTIMATE_GAS, self.estimate_gas(None, address, None, Some("0x"))).await;

        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        recorder.record_optional_available_timed(method::TRACE_CALL, self.trace_call(&transaction)).await
    }
}
