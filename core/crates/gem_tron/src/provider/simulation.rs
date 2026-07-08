use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainSimulation;
use gem_client::Client;
use primitives::{AssetId, Chain, SimulationHeader, SimulationInput, SimulationResult};

use crate::{models::TriggerSmartContractData, rpc::client::TronClient};

#[async_trait]
impl<C: Client + Clone> ChainSimulation for TronClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let signer_address = input.signer_address.as_deref().unwrap_or_default();
        let Some(contract_data) = TriggerSmartContractData::from_payload(Some(input.encoded_transaction.as_bytes()), signer_address)? else {
            return Ok(SimulationResult::default());
        };
        let Some(call_value) = contract_data.call_value.filter(|value| *value > 0) else {
            return Ok(SimulationResult::default());
        };

        Ok(SimulationResult {
            header: Some(SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Tron),
                value: call_value.to_string(),
                is_unlimited: false,
            }),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::trongrid::client::TronGridClient;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn simulate_trigger_smart_contract_surfaces_call_value_as_header() {
        let mock = MockClient::new();
        let client = TronClient::new(mock.clone(), TronGridClient::new(mock, String::new()));
        let input = SimulationInput::new(include_str!("../../testdata/wallet_connect_trigger_smart_contract.json"));

        let result = client.simulate_transaction(input).await.unwrap();
        assert_eq!(
            result.header,
            Some(SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Tron),
                value: "27334102".to_string(),
                is_unlimited: false,
            })
        );
        assert!(result.balance_changes.is_empty());
        assert!(result.payload.is_empty());
    }
}
