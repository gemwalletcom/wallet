use std::{collections::BTreeMap, fmt::Display};

use async_trait::async_trait;
use primitives::{NodeCheckReport, NodeCheckRequest, NodeCheckStatus, NodeSyncStatus};

use crate::ChainState;

const MAX_RESULT_LENGTH: usize = 128;

pub(crate) async fn check_node<T: ChainState + ?Sized>(state: &T, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
    let recorder = record_node_state(state, status, None, NodeCheckRecorder::new(), "chain_id", "latest_block_number").await;
    if recorder.has_failed() {
        return recorder.finish();
    }
    let recorder = match request {
        NodeCheckRequest::Basic => recorder,
        NodeCheckRequest::Wallet { .. } | NodeCheckRequest::Parser { .. } => recorder.record_error("node_check", "profile not supported"),
    };
    recorder.finish()
}

pub struct NodeCheckRecorder {
    checks: BTreeMap<String, NodeCheckStatus>,
}

impl NodeCheckRecorder {
    fn new() -> Self {
        Self { checks: BTreeMap::new() }
    }

    pub fn record<T: Display, E: Display>(self, method: &str, result: Result<T, E>) -> Self {
        let (recorder, _) = self.record_value(method, result);
        recorder
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
        let (recorder, _) = self.record_result(method, result.map(|value| (value, "available".to_string())));
        recorder
    }

    fn record_error(self, method: &str, error: impl Display) -> Self {
        self.record_status(method, NodeCheckStatus::Failed { error: error.to_string() })
    }

    fn has_failed(&self) -> bool {
        self.checks.values().any(|status| match status {
            NodeCheckStatus::Passed { .. } => false,
            NodeCheckStatus::Failed { .. } => true,
        })
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
        let checks = self.checks.into_iter().chain([(method.to_string(), status)]).collect();
        Self { checks }
    }
}

#[async_trait]
pub trait ChainNodeStatus: ChainState {
    async fn get_node_status(&self, request: &NodeCheckRequest, status: &NodeSyncStatus) -> NodeCheckReport {
        let recorder = self.get_node_basic_status(status, NodeCheckRecorder::new()).await;
        if recorder.has_failed() {
            return recorder.finish();
        }

        let recorder = match request {
            NodeCheckRequest::Basic => recorder,
            NodeCheckRequest::Wallet { address, transaction_id } => self.get_node_wallet_status(address, transaction_id, recorder).await,
            NodeCheckRequest::Parser { address, transaction_id } => self.get_node_parser_status(address, transaction_id, recorder).await,
        };
        recorder.finish()
    }

    async fn get_node_basic_status(&self, status: &NodeSyncStatus, recorder: NodeCheckRecorder) -> NodeCheckRecorder;

    async fn get_node_wallet_status(&self, address: &str, transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder;

    async fn get_node_parser_status(&self, address: &str, transaction_id: &str, recorder: NodeCheckRecorder) -> NodeCheckRecorder;
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
    recorder.record(block_number_method, block_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_large_results() {
        let recorder = NodeCheckRecorder::new();
        let value = "x".repeat(MAX_RESULT_LENGTH + 1);
        let result: Result<String, &str> = Ok(value.clone());

        let (recorder, recorded) = recorder.record_value("method", result);
        assert_eq!(recorded, Some(value));
        assert_eq!(recorder.finish().checks.get("method"), Some(&NodeCheckStatus::Passed { result: "available".to_string() }));
    }
}
