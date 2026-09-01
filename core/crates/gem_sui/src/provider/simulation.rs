use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainSimulation, ChainToken};
use futures::future::join_all;
use primitives::{Asset, Chain, SimulationBalanceChange, SimulationInput, SimulationResult};

use crate::decode_transaction;
use crate::provider::simulation_mapper::map_simulation_result;
use crate::rpc::SuiProvider;
use crate::tx_builder::{finish_transaction_json_from_sender, is_transaction_json};
use gem_encoding::encode_base64;

#[async_trait]
impl ChainSimulation for SuiProvider {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let encoded_transaction = if is_transaction_json(input.encoded_transaction.as_bytes()) {
            encode_base64(&finish_transaction_json_from_sender(self, &input.encoded_transaction).await?.tx_data)
        } else {
            input.encoded_transaction
        };
        let transaction: sui_types::Transaction = decode_transaction(&encoded_transaction).map_err(|err| format!("parse transaction: {err}"))?;
        let sender = transaction.sender.to_string();

        let simulated = self.simulate_encoded_transaction(&encoded_transaction).await?;
        let mut result = map_simulation_result(&sender, &simulated);

        let changes = std::mem::take(&mut result.balance_changes);
        let assets = self.get_balance_change_assets(&changes).await;
        result.balance_changes = changes
            .into_iter()
            .zip(assets)
            .map(|(change, asset)| match asset {
                Some(asset) => change.with_asset(asset),
                None => change,
            })
            .collect();

        Ok(result)
    }
}

impl SuiProvider {
    async fn get_balance_change_assets(&self, changes: &[SimulationBalanceChange]) -> Vec<Option<Asset>> {
        join_all(changes.iter().map(|change| async move {
            match &change.asset_id.token_id {
                None => Some(Asset::from_chain(Chain::Sui)),
                Some(coin_type) => self.get_token_data(coin_type.clone()).await.ok(),
            }
        }))
        .await
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_ADDRESS_EMPTY, create_sui_test_client};
    use crate::transfer_builder::build_transfer_message_bytes;
    use num_bigint::BigInt;
    use primitives::AssetId;

    #[tokio::test]
    async fn test_simulate_transaction_native_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let encoded_transaction = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS_EMPTY, 100, None).await?;

        let result = client.simulate_transaction(SimulationInput::new(encoded_transaction)).await?;

        assert!(result.warnings.is_empty());
        let change = result
            .balance_changes
            .iter()
            .find(|change| change.asset_id == AssetId::from_chain(Chain::Sui))
            .ok_or("missing sender SUI balance change")?;
        assert!(change.value < BigInt::ZERO);
        assert_eq!(change.symbol.as_deref(), Some("SUI"));
        Ok(())
    }
}
