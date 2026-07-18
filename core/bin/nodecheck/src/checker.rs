use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use chain_traits::{ChainProvider, ChainState};
use gem_client::Client;
use gem_jsonrpc::{
    client::JsonRpcClient,
    types::{ERROR_CLIENT_ERROR, ERROR_METHOD_NOT_FOUND, JsonRpcError},
};

use crate::fixtures::NodeFixture;

pub(crate) enum NodeCheckStatus {
    Passed(String),
    Failed(String),
}

pub(crate) struct NodeCheckMethod {
    pub(crate) method: &'static str,
    pub(crate) status: NodeCheckStatus,
}

pub(crate) trait NodeCheckReporter: Send + Sync {
    fn report(&self, method: NodeCheckMethod);
}

pub(crate) type NodeCheckResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) fn method_result<T, E: Display>(
    reporter: &dyn NodeCheckReporter,
    method: &'static str,
    result: Result<T, E>,
    format_result: impl FnOnce(&T) -> String,
) -> NodeCheckResult<T> {
    match result {
        Ok(value) => {
            reporter.report(NodeCheckMethod {
                method,
                status: NodeCheckStatus::Passed(format_result(&value)),
            });
            Ok(value)
        }
        Err(error) => {
            let error = error.to_string();
            reporter.report(NodeCheckMethod {
                method,
                status: NodeCheckStatus::Failed(error.clone()),
            });
            Err(error.into())
        }
    }
}

pub(crate) async fn check_chain<C>(client: &C, chain_id_method: &'static str, block_number_method: &'static str, reporter: &dyn NodeCheckReporter) -> NodeCheckResult
where
    C: ChainProvider + ChainState,
{
    let chain = client.get_chain();
    let expected = chain.network_id();
    let chain_id = client.get_chain_id().await.map_err(|error| error.to_string()).and_then(|chain_id| {
        if chain_id == expected {
            Ok(chain_id)
        } else {
            Err(format!("expected {expected}, received {chain_id}"))
        }
    });
    method_result(reporter, chain_id_method, chain_id, Clone::clone)?;

    let block_number = client
        .get_block_latest_number()
        .await
        .map_err(|error| error.to_string())
        .and_then(|block_number| if block_number > 0 { Ok(block_number) } else { Err("received zero".to_string()) });
    method_result(reporter, block_number_method, block_number, ToString::to_string)?;
    Ok(())
}

pub(crate) async fn check_batch<C: Client + Clone>(
    client: &JsonRpcClient<C>,
    method: &'static str,
    params: serde_json::Value,
    reporter: &dyn NodeCheckReporter,
) -> NodeCheckResult {
    let calls = vec![(method.to_string(), params.clone()), (method.to_string(), params)];
    let batch = client.batch_call::<serde_json::Value>(calls).await.and_then(|results| results.take_all());
    method_result(reporter, "json_rpc_batch", batch, |results| format!("{} responses", results.len()))?;
    Ok(())
}

pub(crate) fn check_expected_rpc_error<T>(reporter: &dyn NodeCheckReporter, method: &'static str, result: Result<T, JsonRpcError>) -> NodeCheckResult {
    let result = match result {
        Ok(_) => Err("invalid request was accepted".to_string()),
        Err(error) => match error.code {
            ERROR_METHOD_NOT_FOUND | ERROR_CLIENT_ERROR => Err(error.to_string()),
            _ => Ok(error.code),
        },
    };
    method_result(reporter, method, result, |code| format!("expected error {code}"))?;
    Ok(())
}

#[async_trait]
pub(crate) trait NodeCheck: Send + Sync {
    async fn check_load_balancer(&self, reporter: &dyn NodeCheckReporter) -> NodeCheckResult;
    async fn check_indexer(&self, fixture: NodeFixture, reporter: &dyn NodeCheckReporter) -> NodeCheckResult;
}
