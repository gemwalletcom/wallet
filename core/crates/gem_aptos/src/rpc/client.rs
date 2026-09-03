use std::error::Error;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_client::{Client, ClientError, ClientExt};
use num_bigint::BigUint;
use primitives::chain::Chain;
use primitives::{StakeType, TransactionInputType, TransactionLoadInput};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::models::{
    Account, Block, DelegationPoolStake, GasFee, Ledger, Resource, SimulateTransactionQuery, StakingConfig, Transaction, TransactionPayload, TransactionResponse,
    TransactionSignature, TransactionSimulation, ValidatorSet, ViewRequest,
};
use crate::provider::payload_builder::{
    build_stake_transaction_payload, build_swap_transaction_payload, build_token_transfer_transaction_payload, build_transfer_transaction_payload,
    build_unstake_transaction_payload, build_withdraw_transaction_payload,
};
use crate::rpc::target::AptosTarget;
use crate::{DEFAULT_MAX_GAS_AMOUNT, DEFAULT_SWAP_MAX_GAS_AMOUNT};

#[derive(Debug)]
pub struct AptosClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> AptosClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Aptos }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn send<R: DeserializeOwned + Send>(&self, target: AptosTarget) -> Result<R, ClientError> {
        let path = target.path();
        let headers = target.headers();
        match target {
            AptosTarget::SimulateTransaction { simulation, .. } => self.client.post(&path, &simulation).headers(headers).await,
            AptosTarget::SubmitTransaction { transaction } => self.client.post(&path, &transaction).headers(headers).await,
            AptosTarget::View { request } => self.client.post(&path, &request).headers(headers).await,
            AptosTarget::GetLedger
            | AptosTarget::GetBlock { .. }
            | AptosTarget::GetAccount { .. }
            | AptosTarget::GetAccountTransactions { .. }
            | AptosTarget::GetAccountResource { .. }
            | AptosTarget::GetAccountBalance { .. }
            | AptosTarget::GetTransaction { .. }
            | AptosTarget::GetGasPrice => self.client.get(&path).await,
        }
    }

    pub async fn view<R: DeserializeOwned + Send>(&self, request: ViewRequest) -> Result<R, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::View { request }).await?)
    }

    pub async fn get_ledger(&self) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetLedger).await?)
    }

    pub async fn get_block_transactions(&self, block_number: u64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetBlock { height: block_number }).await?)
    }

    pub async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetAccountTransactions { address }).await?)
    }

    pub async fn get_account_resource<T: Serialize + DeserializeOwned + Send>(&self, address: String, resource: &str) -> Result<Resource<T>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .send(AptosTarget::GetAccountResource {
                address,
                resource: resource.to_string(),
            })
            .await?)
    }

    pub async fn get_account_balance(&self, address: &str, asset_type: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self
            .send(AptosTarget::GetAccountBalance {
                address: address.to_string(),
                asset_type: asset_type.to_string(),
            })
            .await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetAccount { address: address.to_string() }).await?)
    }

    pub async fn submit_transaction(&self, transaction: Vec<u8>) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        let response: TransactionResponse = self.send(AptosTarget::SubmitTransaction { transaction }).await?;
        if let Some(message) = response.message {
            return Err(message.into());
        }
        Ok(response)
    }

    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetTransaction { hash: hash.to_string() }).await?)
    }

    pub async fn get_gas_price(&self) -> Result<GasFee, Box<dyn Error + Send + Sync>> {
        Ok(self.send(AptosTarget::GetGasPrice).await?)
    }

    pub async fn calculate_gas_limit(&self, input: &TransactionLoadInput) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let sequence = input.metadata.get_sequence()?;

        match &input.input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::Deposit(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _) => {
                let payload = match &asset.id.token_id {
                    None => build_transfer_transaction_payload(&input.destination_address, &input.value.to_string()),
                    Some(token_id) => build_token_transfer_transaction_payload(token_id, &input.destination_address, &input.value.to_string())?,
                };

                self.simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                    .await
            }
            TransactionInputType::Swap(asset, _, swap_data) => match &swap_data.data.gas_limit {
                Some(gas_limit) => gas_limit.parse::<u64>().map_err(|_| "Invalid Aptos gas limit".into()),
                None => {
                    let payload = build_swap_transaction_payload(&asset.id.token_id, &swap_data.data)?;
                    Ok(self
                        .simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                        .await
                        .unwrap_or(DEFAULT_SWAP_MAX_GAS_AMOUNT))
                }
            },
            TransactionInputType::Stake(_, stake_type) => {
                let payload = match stake_type {
                    StakeType::Stake(validator) => Some(build_stake_transaction_payload(&validator.id, &input.value.to_string())),
                    StakeType::Unstake(delegation) => Some(build_unstake_transaction_payload(&delegation.validator.id, &input.value.to_string())),
                    StakeType::Withdraw(delegation) => Some(build_withdraw_transaction_payload(&delegation.validator.id, &input.value.to_string())),
                    StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => None,
                };

                let payload = payload.ok_or("Unsupported Aptos stake type")?;
                self.simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                    .await
            }
            TransactionInputType::Generic(_, _, _) => Ok(DEFAULT_MAX_GAS_AMOUNT),
            TransactionInputType::TokenApprove(_, _) | TransactionInputType::Perpetual(_, _) | TransactionInputType::Earn(_, _, _) => {
                Err("Unsupported Aptos transaction type".into())
            }
        }
    }

    pub async fn simulate_transaction(&self, sender: &str, sequence: u64, payload: TransactionPayload, gas_price: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let expiration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 1_000_000;
        let query = SimulateTransactionQuery {
            estimate_max_gas_amount: true,
            estimate_gas_unit_price: false,
            estimate_prioritized_gas_unit_price: false,
        };
        let simulation = TransactionSimulation {
            expiration_timestamp_secs: expiration.to_string(),
            gas_unit_price: gas_price.to_string(),
            max_gas_amount: DEFAULT_MAX_GAS_AMOUNT.to_string(),
            payload,
            sender: sender.to_string(),
            sequence_number: sequence.to_string(),
            signature: TransactionSignature::no_account(),
        };

        let response: Vec<Transaction> = self
            .send(AptosTarget::SimulateTransaction {
                simulation: Box::new(simulation),
                query,
            })
            .await?;
        let transaction = response.into_iter().next().ok_or("No simulation result")?;

        transaction.gas_used.ok_or_else(|| "No gas used in simulation".into())
    }

    pub async fn get_validator_set(&self) -> Result<ValidatorSet, Box<dyn Error + Send + Sync>> {
        Ok(self.get_account_resource::<ValidatorSet>("0x1".to_string(), "0x1::stake::ValidatorSet").await?.data)
    }

    pub async fn get_staking_config(&self) -> Result<StakingConfig, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_account_resource::<StakingConfig>("0x1".to_string(), "0x1::staking_config::StakingConfig")
            .await?
            .data)
    }

    pub async fn get_delegation_pool_stake(&self, pool_address: &str, delegator_address: &str) -> Result<DelegationPoolStake, Box<dyn Error + Send + Sync>> {
        let (active, inactive, pending_inactive): (String, String, String) = self.view(ViewRequest::delegation_pool_stake(pool_address, delegator_address)).await?;

        Ok(DelegationPoolStake {
            active: BigUint::from_str(&active).unwrap_or_else(|_| BigUint::from(0u32)),
            inactive: BigUint::from_str(&inactive).unwrap_or_else(|_| BigUint::from(0u32)),
            pending_inactive: BigUint::from_str(&pending_inactive).unwrap_or_else(|_| BigUint::from(0u32)),
        })
    }

    pub async fn get_delegation_for_pool(&self, delegator_address: &str, pool_address: &str) -> Result<(String, DelegationPoolStake), Box<dyn Error + Send + Sync>> {
        let stake = self.get_delegation_pool_stake(pool_address, delegator_address).await?;
        Ok((pool_address.to_string(), stake))
    }

    pub async fn get_operator_commission_percentage(&self, pool_address: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let result: Vec<String> = self.view(ViewRequest::operator_commission_percentage(pool_address)).await?;
        let commission_bps = result.first().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        Ok(commission_bps as f64 / 100.0)
    }

    pub async fn get_stake_lockup_secs(&self, pool_address: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let result: Vec<String> = self.view(ViewRequest::stake_lockup_secs(pool_address)).await?;
        let lockup_secs = result.first().and_then(|s| s.parse::<u64>().ok()).ok_or("Failed to parse lockup_secs")?;

        Ok(lockup_secs)
    }
}

#[cfg(feature = "rpc")]
mod chain_trait_impls {
    use super::*;
    use async_trait::async_trait;
    use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual};

    #[async_trait]
    impl<C: Client> ChainAccount for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainPerpetual for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainAddressStatus for AptosClient<C> {}
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_client::{CONTENT_TYPE, ContentType};
    use serde_json::{Value, json};

    use super::*;

    #[tokio::test]
    async fn test_submit_transaction() {
        let client = AptosClient::new(MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/v1/transactions");
            assert_eq!(body, b"[1,2,3]");
            assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationAptosBcs.as_str()));
            Ok(br#"{"hash":"0xhash"}"#.to_vec())
        }));
        let response = client.submit_transaction(vec![1, 2, 3]).await.unwrap();
        assert_eq!(
            response,
            TransactionResponse {
                hash: Some("0xhash".to_string()),
                message: None,
                error_code: None,
                vm_error_code: None,
            }
        );

        let client = AptosClient::new(MockClient::new().with_post(|_, _| Ok(br#"{"message":"Transaction already in mempool","error_code":"mempool"}"#.to_vec())));
        let error = client.submit_transaction(vec![1, 2, 3]).await.unwrap_err();
        assert_eq!(error.to_string(), "Transaction already in mempool");
    }

    #[tokio::test]
    async fn test_get_delegation_pool_stake() {
        let client = AptosClient::new(MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/v1/view");
            assert_eq!(
                serde_json::from_slice::<Value>(body).unwrap(),
                json!({"function": "0x1::delegation_pool::get_stake", "type_arguments": [], "arguments": ["0xpool", "0xdelegator"]})
            );
            Ok(br#"["1000","200","30"]"#.to_vec())
        }));
        let stake = client.get_delegation_pool_stake("0xpool", "0xdelegator").await.unwrap();
        assert_eq!(
            stake,
            DelegationPoolStake {
                active: BigUint::from(1000u32),
                inactive: BigUint::from(200u32),
                pending_inactive: BigUint::from(30u32),
            }
        );
    }
}
