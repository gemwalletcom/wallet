use async_trait::async_trait;
use chain_traits::{
    ChainBalances, ChainTraits,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::{NodeCheckReport, NodeCheckRequest, NodeSyncStatus};

use crate::{jsonrpc::TransactionObject, method, rpc::EthereumProvider};

#[async_trait]
impl<C: Client + Clone> ChainTraits for EthereumProvider<C> {
    async fn check_node(&self, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
        ChainNodeStatus::get_node_status(self, request, status).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for EthereumProvider<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(self, status, Some(self.get_chain().network_id()), recorder, method::ETH_CHAIN_ID, method::ETH_BLOCK_NUMBER).await
    }

    async fn get_node_wallet_status(&self, address: &str, transaction_id: Option<&str>, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let Some(transaction_id) = transaction_id else {
            return recorder.record("wallet", Err::<&str, _>("missing transaction fixture"));
        };
        let balance = self.get_balance_coin(address.to_string()).await.map(|result| result.balance.available);
        let recorder = recorder.record(method::ETH_GET_BALANCE, balance);

        let recorder = recorder.record(method::ETH_GET_TRANSACTION_COUNT, self.get_transaction_count(address).await);
        let receipt = self
            .get_transaction_receipt(transaction_id)
            .await
            .map_err(|error| error.to_string())
            .and_then(|receipt| receipt.map(|receipt| receipt.block_number).ok_or_else(|| "returned null".to_string()));
        let recorder = recorder.record(method::ETH_GET_TRANSACTION_RECEIPT, receipt);
        let recorder = recorder.record_available(method::ETH_FEE_HISTORY, self.get_fee_history(1, vec![50]).await);
        let recorder = recorder.record(method::ETH_GAS_PRICE, self.get_gas_price().await);
        let recorder = recorder.record(method::ETH_GET_CODE, self.get_code(address).await);
        let recorder = recorder.record(method::ETH_SYNCING, self.get_syncing().await);
        let recorder = recorder.record_available(method::ETH_CALL, self.eth_call(address, &[]).await);
        let recorder = recorder.record(method::ETH_ESTIMATE_GAS, self.estimate_gas(None, address, None, Some("0x")).await);

        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        recorder.record_optional_available(method::TRACE_CALL, self.trace_call(&transaction).await)
    }
}
