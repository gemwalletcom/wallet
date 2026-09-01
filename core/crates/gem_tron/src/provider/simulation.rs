use num_bigint::BigUint;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainSimulation;
use futures::future::join_all;
use gem_client::Client;
use primitives::{Asset, AssetId, Chain, SimulationHeader, SimulationInput, SimulationResult};

use crate::address::TronAddress;
use crate::models::TriggerSmartContractData;
use crate::provider::simulation_mapper::map_simulation_result;
use crate::rpc::TronProvider;

#[async_trait]
impl<C: Client> ChainSimulation for TronProvider<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let signer_address = input.signer_address.as_deref().unwrap_or_default();
        let Some(contract_data) = TriggerSmartContractData::from_payload(Some(input.encoded_transaction.as_bytes()), signer_address)? else {
            return Ok(SimulationResult::default());
        };

        let owner = TronAddress::from_hex_or_base58(&contract_data.owner_address).ok_or("invalid owner address")?;
        let call_value = contract_data.call_value.filter(|value| *value > 0);

        let response = self.trigger_smart_contract_call(&contract_data).await?;
        let SimulationResult {
            warnings,
            balance_changes,
            payload,
            ..
        } = map_simulation_result(&owner, &response, call_value);

        let assets = join_all(balance_changes.iter().map(|change| async move {
            match &change.asset_id.token_id {
                None => Some(Asset::from_chain(Chain::Tron)),
                Some(token_id) => self.get_token_data(token_id.clone()).await.ok(),
            }
        }))
        .await;
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
            header: call_value.map(|value| SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Tron),
                value: Some(BigUint::from(value)),
                is_unlimited: false,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::TronClient;
    use gem_client::testkit::MockClient;
    use num_bigint::BigInt;
    use primitives::{Address as _, SimulationBalanceChange};

    #[tokio::test]
    async fn test_simulate_transaction_surfaces_call_value_as_header() {
        let mock = MockClient::new().with_post(|_, _| Ok(br#"{"result":{"result":true},"constant_result":[],"energy_used":100}"#.to_vec()));
        let client = TronProvider::new_rpc_only(TronClient::new(mock));
        let input = SimulationInput::new(include_str!("../../testdata/wallet_connect_trigger_smart_contract.json"));

        let result = client.simulate_transaction(input).await.unwrap();

        assert_eq!(
            result.header,
            Some(SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Tron),
                value: Some(BigUint::from(27_334_102u32)),
                is_unlimited: false,
            })
        );
        assert_eq!(
            result.balance_changes,
            vec![SimulationBalanceChange::new(AssetId::from_chain(Chain::Tron), BigInt::from(-27_334_102)).with_asset(Asset::from_chain(Chain::Tron))]
        );
    }

    #[tokio::test]
    async fn test_simulate_transaction_decodes_swap_logs() {
        let fixture = include_str!("../../testdata/trigger_constant_contract_swap_with_logs.json");
        let mock = MockClient::new().with_post(move |_, _| Ok(fixture.as_bytes().to_vec()));
        let client = TronProvider::new_rpc_only(TronClient::new(mock));
        let encoded_transaction = include_str!("../../testdata/wallet_connect_swap_trigger_smart_contract.json");

        let result = ChainSimulation::simulate_transaction(&client, SimulationInput::new(encoded_transaction)).await.unwrap();

        assert!(result.warnings.is_empty());
        assert_eq!(
            result.header,
            Some(SimulationHeader {
                asset_id: AssetId::from_chain(Chain::Tron),
                value: Some(BigUint::from(1_000_000u32)),
                is_unlimited: false,
            })
        );
        assert_eq!(result.balance_changes.len(), 2);
        assert_eq!(result.balance_changes[0].asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(result.balance_changes[0].value, BigInt::from(-1000000i64));
        let output_token = TronAddress::from_hex("4e4bee11cea0070f957b98fd8cf4138ef3295e0e").unwrap().encode();
        assert_eq!(result.balance_changes[1].asset_id, AssetId::from_token(Chain::Tron, &output_token));
        assert_eq!(result.balance_changes[1].value, BigInt::from(329114i64));
    }

    #[tokio::test]
    async fn test_simulate_transaction_reverted_contract_call_returns_warning() {
        let mock = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/trigger_constant_contract_reverted.json").as_bytes().to_vec()));
        let client = TronProvider::new_rpc_only(TronClient::new(mock));
        let encoded_transaction = include_str!("../../testdata/wallet_connect_swap_trigger_smart_contract.json");

        let result = ChainSimulation::simulate_transaction(&client, SimulationInput::new(encoded_transaction)).await.unwrap();

        assert_eq!(result.warnings.len(), 1);
        assert!(result.balance_changes.is_empty());
    }

    #[tokio::test]
    async fn test_simulate_transaction_non_trigger_smart_contract_returns_default() {
        let mock = MockClient::new().with_post(|_, _| Ok(Vec::new()));
        let client = TronProvider::new_rpc_only(TronClient::new(mock));
        let encoded_transaction = include_str!("../../testdata/wallet_connect_vote_witness_contract.json");

        let result = ChainSimulation::simulate_transaction(&client, SimulationInput::new(encoded_transaction)).await.unwrap();

        assert_eq!(result, SimulationResult::default());
    }
}
