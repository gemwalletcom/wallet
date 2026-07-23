#![cfg(feature = "rpc")]

use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainSimulation, ChainToken};
use futures::future::join_all;
use gem_client::Client;
use primitives::{Asset, SimulationBalanceChange, SimulationInput, SimulationResult};

use crate::jsonrpc::TransactionObject;
use crate::provider::simulation_mapper::map_simulation_result;
use crate::rpc::EthereumProvider;

#[async_trait]
impl<C: Client + Clone> ChainSimulation for EthereumProvider<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let transaction: TransactionObject = serde_json::from_str(&input.encoded_transaction)?;
        let signer = transaction.from.as_deref().filter(|from| !from.is_empty()).ok_or("missing sender address")?;

        let trace = self.trace_call(&transaction).await?;
        let SimulationResult {
            warnings,
            balance_changes,
            payload,
            header,
        } = map_simulation_result(self.get_chain(), signer, &trace);

        let assets = self.get_balance_change_assets(&balance_changes).await;
        let balance_changes = balance_changes
            .into_iter()
            .zip(assets)
            .map(|(change, asset)| match asset {
                Some(asset) => change.with_asset(asset),
                None => change,
            })
            .collect();

        Ok(SimulationResult {
            warnings,
            balance_changes,
            payload,
            header,
        })
    }
}

impl<C: Client + Clone> EthereumProvider<C> {
    async fn get_balance_change_assets(&self, changes: &[SimulationBalanceChange]) -> Vec<Option<Asset>> {
        join_all(changes.iter().map(|change| async move {
            match &change.asset_id.token_id {
                None => Some(Asset::from_chain(self.get_chain())),
                Some(token_id) => self.get_token_data(token_id.clone()).await.ok(),
            }
        }))
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method;
    use crate::rpc::EthereumClient;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID;
    use primitives::testkit::json_rpc::load_json_rpc_result;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use primitives::{AssetId, Chain, EVMChain};
    use serde_json::Value;

    #[tokio::test]
    async fn test_simulate_transaction_native_transfer() {
        let ethereum_client = mock_jsonrpc_client(|request_method, _| match request_method {
            method::TRACE_CALL => Ok(load_json_rpc_result(include_str!("../../testdata/trace_call_native_transfer.json"))),
            _ => Ok(Value::Null),
        });
        let ethereum_client = EthereumProvider::new_rpc_only(EthereumClient::new(ethereum_client, EVMChain::Ethereum));

        let encoded_transaction = serde_json::to_string(&TransactionObject::mock(TEST_EVM_RECIPIENT, Some("0x2386f26fc10000"))).unwrap();
        let result = ChainSimulation::simulate_transaction(&ethereum_client, SimulationInput::new(encoded_transaction))
            .await
            .unwrap();

        assert!(result.warnings.is_empty());
        assert_eq!(
            result.balance_changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                value: "-10000000000000000".to_string(),
                decimals: 18,
                name: Some("Ethereum".to_string()),
                symbol: Some("ETH".to_string()),
            }]
        );
    }

    #[tokio::test]
    async fn test_simulate_transaction_root_revert_returns_validation_warning() {
        let trace_result: Value = load_json_rpc_result(include_str!("../../testdata/trace_call_reverted_root.json"));
        let ethereum_client = mock_jsonrpc_client(move |request_method, _| match request_method {
            method::TRACE_CALL => Ok(trace_result.clone()),
            _ => Ok(Value::Null),
        });
        let ethereum_client = EthereumProvider::new_rpc_only(EthereumClient::new(ethereum_client, EVMChain::Ethereum));

        let encoded_transaction = serde_json::to_string(&TransactionObject::mock(TEST_EVM_RECIPIENT, None)).unwrap();
        let result = ChainSimulation::simulate_transaction(&ethereum_client, SimulationInput::new(encoded_transaction))
            .await
            .unwrap();

        assert_eq!(result.warnings.len(), 1);
        assert!(result.balance_changes.is_empty());
    }

    #[tokio::test]
    async fn test_simulate_transaction_preserves_change_when_token_metadata_lookup_fails() {
        let ethereum_client = mock_jsonrpc_client(|request_method, _| match request_method {
            method::TRACE_CALL => Ok(load_json_rpc_result(include_str!("../../testdata/trace_call_erc20_transfer_proxy.json"))),
            _ => Ok(Value::Null),
        });
        let ethereum_client = EthereumProvider::new_rpc_only(EthereumClient::new(ethereum_client, EVMChain::Ethereum));

        let encoded_transaction = serde_json::to_string(&TransactionObject::mock(ETHEREUM_USDC_TOKEN_ID, None)).unwrap();
        let result = ChainSimulation::simulate_transaction(&ethereum_client, SimulationInput::new(encoded_transaction))
            .await
            .unwrap();

        assert_eq!(
            result.balance_changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                value: "-1000000".to_string(),
                decimals: 0,
                name: None,
                symbol: None,
            }]
        );
    }
}
