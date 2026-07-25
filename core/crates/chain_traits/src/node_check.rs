use std::{collections::BTreeMap, fmt::Display};

use async_trait::async_trait;
use primitives::{NodeCheckReport, NodeCheckRequest, NodeCheckStatus, NodeSyncStatus};

use crate::{ChainBlockTransactions, ChainState};

const MAX_RESULT_LENGTH: usize = 128;
const PARSER_BLOCK_OFFSET: u64 = 10;

pub(crate) async fn check_node<T: ChainBlockTransactions + ChainState + ?Sized>(state: &T, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
    let recorder = record_node_state(state, status, None, NodeCheckRecorder::default(), "chain_id", "latest_block_number").await;
    if recorder.has_failed() {
        return recorder.finish();
    }
    let recorder = match request {
        NodeCheckRequest::Basic => recorder,
        NodeCheckRequest::Wallet { .. } => recorder.record_error("node_check", "profile not supported"),
        NodeCheckRequest::Parser => record_parser_block(state, recorder).await,
    };
    recorder.finish()
}

#[derive(Default)]
pub struct NodeCheckRecorder {
    checks: BTreeMap<String, NodeCheckStatus>,
    block_number: Option<u64>,
}

impl NodeCheckRecorder {
    pub fn record<T: Display, E: Display>(self, method: &str, result: Result<T, E>) -> Self {
        self.record_value(method, result).0
    }

    pub fn record_value<T: Display, E: Display>(self, method: &str, result: Result<T, E>) -> (Self, Option<T>) {
        self.record_result(
            method,
            result.map(|value| {
                let result = value.to_string();
                let result = if result.len() > MAX_RESULT_LENGTH { "available".to_string() } else { result };
                (value, result)
            }),
        )
    }

    pub fn record_available<T, E: Display>(self, method: &str, result: Result<T, E>) -> Self {
        self.record_result(method, result.map(|value| (value, "available".to_string()))).0
    }

    pub fn record_optional_available<T, E: Display>(self, method: &str, result: Result<T, E>) -> Self {
        let status = match result {
            Ok(_) => NodeCheckStatus::Passed { result: "available".to_string() },
            Err(error) => NodeCheckStatus::Warning { warning: error.to_string() },
        };
        self.record_status(method, status)
    }

    fn record_error(self, method: &str, error: impl Display) -> Self {
        self.record_status(method, NodeCheckStatus::Failed { error: error.to_string() })
    }

    fn has_failed(&self) -> bool {
        self.checks.values().any(|status| matches!(status, NodeCheckStatus::Failed { .. }))
    }

    fn finish(self) -> NodeCheckReport {
        NodeCheckReport { checks: self.checks }
    }

    fn record_result<T, E: Display>(self, method: &str, result: Result<(T, String), E>) -> (Self, Option<T>) {
        let (status, value) = match result {
            Ok((value, result)) => (NodeCheckStatus::Passed { result }, Some(value)),
            Err(error) => (NodeCheckStatus::Failed { error: error.to_string() }, None),
        };
        (self.record_status(method, status), value)
    }

    fn record_status(self, method: &str, status: NodeCheckStatus) -> Self {
        let block_number = self.block_number;
        let mut checks = self.checks;
        checks.insert(method.to_string(), status);
        Self { checks, block_number }
    }
}

#[async_trait]
pub trait ChainNodeStatus: ChainBlockTransactions + ChainState {
    async fn get_node_status(&self, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
        let recorder = self.get_node_basic_status(status, NodeCheckRecorder::default()).await;
        if recorder.has_failed() {
            return recorder.finish();
        }

        let recorder = match request {
            NodeCheckRequest::Basic => recorder,
            NodeCheckRequest::Wallet { address, transaction_id } => self.get_node_wallet_status(address, transaction_id, recorder).await,
            NodeCheckRequest::Parser => record_parser_block(self, recorder).await,
        };
        recorder.finish()
    }

    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder;

    async fn get_node_wallet_status(&self, address: &str, transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder;
}

async fn record_parser_block<T: ChainBlockTransactions + ?Sized>(state: &T, recorder: NodeCheckRecorder) -> NodeCheckRecorder {
    let Some(latest_block) = recorder.block_number else {
        return recorder;
    };

    let block_number = latest_block.saturating_sub(PARSER_BLOCK_OFFSET);
    recorder.record(
        "block_transactions",
        state.get_transactions_by_block(block_number).await.map(|transactions| transactions.len()),
    )
}

pub async fn record_node_state<T: ChainState + ?Sized>(
    state: &T,
    status: &NodeSyncStatus,
    expected_chain_id: Option<&str>,
    recorder: NodeCheckRecorder,
    chain_id_method: &str,
    block_number_method: &str,
) -> NodeCheckRecorder {
    let chain_id = state.get_chain_id().await.map_err(|error| error.to_string()).and_then(|chain_id| match expected_chain_id {
        Some(expected) if chain_id != expected => Err(format!("expected {expected}, received {chain_id}")),
        _ => Ok(chain_id),
    });
    let (recorder, chain_id) = recorder.record_value(chain_id_method, chain_id);
    if chain_id.is_none() {
        return recorder;
    }

    let block_number = match status.current_block_number.or(status.latest_block_number) {
        Some(block_number) => Ok(block_number),
        None => state.get_block_latest_number().await.map_err(|error| error.to_string()),
    }
    .and_then(|block_number| if block_number > 0 { Ok(block_number) } else { Err("received zero".to_string()) });
    let (recorder, block_number) = recorder.record_value(block_number_method, block_number);
    NodeCheckRecorder { block_number, ..recorder }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use primitives::Transaction;

    use super::*;
    use crate::ChainBlockTransactions;

    #[derive(Default)]
    struct ParserState {
        latest_block_calls: AtomicUsize,
        block_transaction_calls: AtomicUsize,
    }

    #[async_trait]
    impl ChainBlockTransactions for ParserState {
        async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
            self.block_transaction_calls.fetch_add(1, Ordering::Relaxed);
            assert_eq!(block, 90);
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl ChainState for ParserState {
        async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
            Ok("chain".to_string())
        }

        async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
            self.latest_block_calls.fetch_add(1, Ordering::Relaxed);
            Ok(100)
        }
    }

    #[test]
    fn test_parser_profile_checks_block_transactions() {
        let state = ParserState::default();
        let report = futures::executor::block_on(check_node(&state, &NodeCheckRequest::Parser, &NodeSyncStatus::in_sync()));

        assert_eq!(report.checks.get("block_transactions"), Some(&NodeCheckStatus::Passed { result: "0".to_string() }));
        assert_eq!(report.checks.len(), 3);
        assert_eq!(state.latest_block_calls.load(Ordering::Relaxed), 1);
        assert_eq!(state.block_transaction_calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn hides_large_results() {
        let recorder = NodeCheckRecorder::default();
        let value = "x".repeat(MAX_RESULT_LENGTH + 1);
        let result: Result<String, &str> = Ok(value.clone());

        let (recorder, recorded) = recorder.record_value("method", result);
        assert_eq!(recorded, Some(value));
        assert_eq!(recorder.finish().checks.get("method"), Some(&NodeCheckStatus::Passed { result: "available".to_string() }));
    }

    #[test]
    fn records_optional_failure_as_warning() {
        let result: Result<(), &str> = Err("method not found");
        let recorder = NodeCheckRecorder::default().record_optional_available("method", result);

        assert_eq!(
            recorder.finish().checks.get("method"),
            Some(&NodeCheckStatus::Warning {
                warning: "method not found".to_string()
            })
        );
    }
}
