use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainSimulation;
use gem_client::Client;
use primitives::{SimulationInput, SimulationResult};

use crate::{
    Address,
    models::{simulation::TonEmulationRequest, wallet_connect::TonConnectRequest},
    provider::simulation_mapper::map_simulation_result,
    rpc::client::TonClient,
};

#[async_trait]
impl<C: Client> ChainSimulation for TonClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let request: TonConnectRequest = serde_json::from_str(&input.encoded_transaction)?;
        request.validate_for_emulation()?;
        let request_sender = request.from.as_deref().filter(|address| !address.is_empty());
        let input_sender = input.signer_address.as_deref().filter(|address| !address.is_empty());
        if let Some(input_sender) = input_sender {
            Address::ensure_matches(request_sender, input_sender)?;
        }
        let sender = input_sender.or(request_sender).ok_or("missing TON sender address")?;
        let sender = Address::parse(sender)?;
        let sender_address = request_sender.map(str::to_owned).unwrap_or_else(|| sender.encode_non_bounceable());
        let emulation_request = TonEmulationRequest {
            from: &sender_address,
            messages: &request.messages,
            valid_until: request.valid_until,
            with_actions: true,
        };
        let response = self.emulate_ton_connect(&emulation_request).await?;
        Ok(map_simulation_result(&sender, response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;
    use num_bigint::BigInt;
    use primitives::{AssetId, Chain, asset_constants::TON_DUST_TOKEN_ID};
    use serde_json::Value;

    #[tokio::test]
    async fn test_simulate_transaction_maps_raw_dedust_balance_changes() {
        let response = include_str!("../../testdata/emulate_ton_connect_dedust_response.json");
        let mock = MockClient::new().with_post_with_headers(move |path, body, headers| {
            assert_eq!(path, "/api/emulate/v1/emulateTonConnect");
            assert_eq!(headers.get("X-Actions-Version").map(String::as_str), Some("5"));
            let request: Value = serde_json::from_slice(body).unwrap();
            let mut expected: Value = serde_json::from_str(include_str!("../../testdata/wallet_connect_dedust_emulation_request.json")).unwrap();
            expected["with_actions"] = Value::Bool(true);
            assert_eq!(request, expected);
            Ok(response.as_bytes().to_vec())
        });
        let client = TonClient::new(mock);
        let input = SimulationInput::new(include_str!("../../testdata/wallet_connect_dedust_emulation_request.json"));

        let result = client.simulate_transaction(input).await.unwrap();

        assert!(result.warnings.is_empty());
        assert_eq!(result.balance_changes.len(), 2);
        assert_eq!(result.balance_changes[0].asset_id, AssetId::from_chain(Chain::Ton));
        assert_eq!(result.balance_changes[0].value, BigInt::from(-1014643500i64));
        assert_eq!(result.balance_changes[1].asset_id, AssetId::from_token(Chain::Ton, TON_DUST_TOKEN_ID));
        assert_eq!(result.balance_changes[1].value, BigInt::from(2228076648i64));
    }

    #[tokio::test]
    async fn test_simulate_transaction_rejects_sender_mismatch() {
        let client = TonClient::new(MockClient::new());
        let input = SimulationInput::new(include_str!("../../testdata/wallet_connect_dedust_emulation_request.json"))
            .with_signer_address("0:44a14a5a9406979d59b9328898591660b8b1736342b11632efdcc911ab9057cf");

        let error = client.simulate_transaction(input).await.unwrap_err();

        assert_eq!(error.to_string(), "Invalid input: TON from does not match signer address");
    }
}
