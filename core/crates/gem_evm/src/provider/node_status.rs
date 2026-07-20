use async_trait::async_trait;
use chain_traits::{
    ChainBalances,
    node_check::{ChainNodeStatus, NodeCheckRecorder, record_node_state},
};
use gem_client::Client;
use primitives::{NodeSyncStatus, NodeType};

use crate::{jsonrpc::TransactionObject, method, rpc::EthereumClient};

#[async_trait]
impl<C: Client + Clone> ChainNodeStatus for EthereumClient<C> {
    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        record_node_state(self, status, Some(self.get_chain().network_id()), recorder, method::ETH_CHAIN_ID, method::ETH_BLOCK_NUMBER).await
    }

    async fn get_node_wallet_status(&self, address: &str, transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
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
        let recorder = recorder.record(method::ETH_GAS_PRICE, self.gas_price().await);
        let recorder = recorder.record(method::ETH_GET_CODE, self.get_code(address).await);
        let recorder = recorder.record(method::ETH_SYNCING, self.get_syncing().await);
        let recorder = recorder.record(method::ETH_CALL, self.eth_call::<String>(address, "0x").await);
        let recorder = recorder.record(method::ETH_ESTIMATE_GAS, self.estimate_gas(None, address, None, Some("0x")).await);

        let transaction = TransactionObject::new_call_with_from(address, address, Vec::new());
        recorder.record_available(method::TRACE_CALL, self.trace_call(&transaction).await)
    }

    async fn get_node_parser_status(&self, _address: &str, transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
        let block_number = self
            .get_transaction(transaction_id)
            .await
            .map_err(|error| error.to_string())
            .and_then(|transaction| transaction.ok_or_else(|| "returned null".to_string()))
            .map(|transaction| transaction.block_number);
        let (recorder, block_number) = recorder.record_value(method::ETH_GET_TRANSACTION_BY_HASH, block_number);
        let Some(block_number) = block_number else {
            return recorder;
        };

        let receipt = self
            .get_transaction_receipt(transaction_id)
            .await
            .map_err(|error| error.to_string())
            .and_then(|receipt| receipt.ok_or_else(|| "returned null".to_string()))
            .map(|receipt| receipt.block_number);
        let recorder = recorder.record(method::ETH_GET_TRANSACTION_RECEIPT, receipt);

        let recorder = recorder.record_available(method::ETH_GET_BLOCK_BY_NUMBER, self.get_block(block_number).await);
        let recorder = recorder.record_available(method::ETH_GET_BLOCK_RECEIPTS, self.get_block_receipts(block_number).await);
        match self.node_type {
            NodeType::Default => recorder,
            NodeType::Archival => {
                let recorder = recorder.record_available(method::TRACE_REPLAY_BLOCK_TRANSACTIONS, self.trace_replay_block_transactions(block_number).await);
                recorder.record_available(method::TRACE_REPLAY_TRANSACTION, self.trace_replay_transaction(transaction_id).await)
            }
        }
    }
}
