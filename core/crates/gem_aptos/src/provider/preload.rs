use async_trait::async_trait;
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use std::error::Error;

use gem_client::Client;
use primitives::{AssetId, Chain, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use super::preload_mapper::map_transaction_preload;
use crate::provider::payload_builder::build_delegation_pool_payload_data;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for AptosClient<C> {
    fn transaction_fee_estimate_units(&self, operation: TransactionFeeOperation) -> Option<u64> {
        Some(match operation {
            TransactionFeeOperation::Transfer | TransactionFeeOperation::TokenTransfer => crate::DEFAULT_MAX_GAS_AMOUNT,
            TransactionFeeOperation::Swap => crate::DEFAULT_SWAP_MAX_GAS_AMOUNT,
        })
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&input.sender_address).await?;
        map_transaction_preload(&account)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let gas_limit = self.calculate_gas_limit(&input).await?;
        let fee = TransactionFee::calculate(gas_limit, &input.gas_price, AssetId::from_chain(Chain::Aptos));

        let data = match &input.input_type {
            TransactionInputType::Stake { stake_type, .. } => build_delegation_pool_payload_data(stake_type, &input.value.to_string())?,
            _ => None,
        };

        let sequence = input.metadata.get_sequence()?;
        let metadata = TransactionLoadMetadata::Aptos { sequence, data };

        Ok(TransactionLoadData { fee, metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_fee = self.get_gas_price().await?;

        Ok(vec![
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(gas_fee.gas_estimate)),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(gas_fee.prioritized_gas_estimate)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;
    use num_bigint::BigUint;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, ChainSigner, Delegation, DelegationValidator, SignerInput, StakeType};
    use serde_json::Value;

    use super::*;
    use crate::KNOWN_VALIDATOR_POOL;
    use crate::models::{SignedTransaction, SubmitTransactionBcsRequest, TransactionPayloadBCS};
    use crate::signer::{AptosChainSigner, EntryFunctionPayload};

    const TEST_TOKEN_ID: &str = "0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b";
    const TEST_TRANSFER_ABI: &[&str] = &["address", "u64"];
    const TEST_FUNGIBLE_TRANSFER_ABI: &[&str] = &["address", "address", "u64"];
    const TEST_DELEGATION_POOL_ABI: &[&str] = &["address", "u64"];

    async fn assert_signs_simulated_payload(input_type: TransactionInputType, abi: &[&str]) {
        let simulated = Arc::new(Mutex::new(None));
        let client = AptosClient::new(MockClient::new().with_post({
            let simulated = simulated.clone();
            move |_, body| {
                *simulated.lock().unwrap() = Some(serde_json::from_slice::<Value>(body).unwrap()["payload"].clone());
                Ok(br#"[{"success":true,"vm_status":"Executed successfully","gas_used":"5075"}]"#.to_vec())
            }
        }));
        let input = TransactionLoadInput {
            input_type,
            value: BigUint::from(1_102_185_008u64),
            ..TransactionLoadInput::mock_aptos_token_transfer(TEST_TOKEN_ID)
        };

        let load = client.get_transaction_load(input.clone()).await.unwrap();
        let input = SignerInput::new(TransactionLoadInput { metadata: load.metadata, ..input }, load.fee);
        let signed = match &input.input_type {
            TransactionInputType::Transfer { asset } if asset.id.token_id.is_none() => AptosChainSigner.sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            TransactionInputType::Transfer { .. } => AptosChainSigner.sign_token_transfer(&input, &TEST_PRIVATE_KEY).unwrap(),
            _ => AptosChainSigner.sign_stake(&input, &TEST_PRIVATE_KEY).unwrap().remove(0),
        };
        let request: SubmitTransactionBcsRequest = serde_json::from_str(&signed).unwrap();
        let signed: SignedTransaction = bcs::from_bytes(&hex::decode(request.bcs).unwrap()).unwrap();

        let simulated = simulated.lock().unwrap().take().unwrap();
        let simulated = serde_json::from_value::<EntryFunctionPayload>(simulated).unwrap().to_entry_function(Some(abi)).unwrap();

        assert_eq!(signed.raw_tx.payload, TransactionPayloadBCS::EntryFunction(simulated));
    }

    #[tokio::test]
    async fn test_get_transaction_load_simulates_the_signed_payload() {
        let pool = DelegationValidator::stake(Chain::Aptos, KNOWN_VALIDATOR_POOL.to_string(), String::new(), true, 0.0, 0.0);
        let asset = Asset::from_chain(Chain::Aptos);

        assert_signs_simulated_payload(TransactionInputType::Transfer { asset: asset.clone() }, TEST_TRANSFER_ABI).await;
        assert_signs_simulated_payload(TransactionLoadInput::mock_aptos_token_transfer(TEST_TOKEN_ID).input_type, TEST_FUNGIBLE_TRANSFER_ABI).await;
        assert_signs_simulated_payload(
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Stake(pool.clone()),
            },
            TEST_DELEGATION_POOL_ABI,
        )
        .await;
        assert_signs_simulated_payload(
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Unstake(Delegation::mock_with_validator(pool.clone())),
            },
            TEST_DELEGATION_POOL_ABI,
        )
        .await;
        assert_signs_simulated_payload(
            TransactionInputType::Stake {
                asset,
                stake_type: StakeType::Withdraw(Delegation::mock_with_validator(pool)),
            },
            TEST_DELEGATION_POOL_ABI,
        )
        .await;
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::KNOWN_VALIDATOR_POOL;
    use crate::provider::testkit::{TEST_ADDRESS_STAKING, create_aptos_test_client};
    use num_bigint::BigInt;
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, DelegationValidator, StakeType};
    use serde_json::Value;

    #[tokio::test]
    async fn test_aptos_get_transaction_load_stake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let metadata = client
            .get_transaction_preload(TransactionPreloadInput {
                input_type: TransactionInputType::Stake {
                    asset: Asset::from_chain(Chain::Aptos),
                    stake_type: StakeType::Stake(DelegationValidator::stake(Chain::Aptos, KNOWN_VALIDATOR_POOL.to_string(), String::new(), true, 0.0, 0.0)),
                },
                sender_address: TEST_ADDRESS_STAKING.to_string(),
                destination_address: KNOWN_VALIDATOR_POOL.to_string(),
                references: vec![],
            })
            .await?;

        let load = client
            .get_transaction_load(TransactionLoadInput {
                input_type: TransactionInputType::Stake {
                    asset: Asset::from_chain(Chain::Aptos),
                    stake_type: StakeType::Stake(DelegationValidator::stake(Chain::Aptos, KNOWN_VALIDATOR_POOL.to_string(), String::new(), true, 0.0, 0.0)),
                },
                sender_address: TEST_ADDRESS_STAKING.to_string(),
                destination_address: KNOWN_VALIDATOR_POOL.to_string(),
                value: BigUint::from(1100000000u64),
                gas_price: GasPriceType::regular(BigInt::from(100u64)),
                memo: None,
                is_max_value: false,
                metadata,
            })
            .await?;

        let TransactionLoadMetadata::Aptos { data: Some(data), .. } = load.metadata else {
            panic!("Expected Aptos transaction load metadata with payload");
        };

        let payload: Value = serde_json::from_str(&data).unwrap();
        assert_eq!(payload["function"], "0x1::delegation_pool::add_stake");
        assert!(load.fee.gas_limit > BigInt::from(0u32));

        Ok(())
    }
}
