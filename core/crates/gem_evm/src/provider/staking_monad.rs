use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::monad::{
    IMonadStakingLens, MONAD_SCALE, MonadLensDelegation, MonadLensValidatorInfo, STAKING_LENS_CONTRACT, decode_get_lens_apys, decode_get_lens_balance, decode_get_lens_delegations,
    decode_get_lens_validators, delegation_id, encode_get_lens_apys, encode_get_lens_balance, encode_get_lens_delegations, encode_get_lens_validators,
};
use crate::rpc::client::EthereumClient;

const MONAD_VALIDATOR_NAMES: &[(u64, &str)] = &[(16, "MonadVision"), (5, "Alchemy"), (10, "Stakin"), (9, "Everstake")];

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub(crate) async fn call_monad_delegations(&self, address: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_delegations(address)?;
        self.call_lens(data).await
    }

    pub async fn get_monad_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_apys(&[]);
        let result = self.call_lens(data).await?;

        let apys = decode_get_lens_apys(&result)?;
        let apy_bps = apys.into_iter().max().unwrap_or(0);

        if apy_bps == 0 {
            return Ok(None);
        }

        Ok(Some(apy_bps as f64 / 100.0))
    }

    pub async fn get_monad_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validator_names: HashMap<u64, &str> = MONAD_VALIDATOR_NAMES.iter().copied().collect();
        let validator_ids = MONAD_VALIDATOR_NAMES.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let data = encode_get_lens_validators(&validator_ids);
        let result = self.call_lens(data).await?;

        let (validators, network_apy_bps) = decode_get_lens_validators(&result)?;
        let network_apy = network_apy_bps as f64 / 100.0;

        Ok(validators
            .into_iter()
            .map(|validator| Self::map_lens_validator(&validator, &validator_names, network_apy))
            .collect())
    }

    pub async fn get_monad_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let positions = match self.call_monad_delegations(address).await {
            Ok(bytes) => match decode_get_lens_delegations(&bytes) {
                Ok(position_list) => position_list,
                Err(_) => return Ok(Vec::new()),
            },
            Err(_) => return Ok(Vec::new()),
        };

        let mut delegations = Vec::new();

        for position in positions {
            if position.amount.is_zero() && position.rewards.is_zero() {
                continue;
            }

            let state = Self::map_lens_state(&position);
            let completion_date = if position.completion_timestamp == 0 {
                None
            } else {
                DateTime::<Utc>::from_timestamp(position.completion_timestamp as i64, 0)
            };

            delegations.push(DelegationBase {
                asset_id: AssetId::from_chain(Chain::Monad),
                state,
                balance: position.amount,
                shares: BigUint::zero(),
                rewards: position.rewards,
                completion_date,
                delegation_id: delegation_id(address, position.validator_id, state, position.withdraw_id),
                validator_id: position.validator_id.to_string(),
            });
        }

        Ok(delegations)
    }

    pub async fn get_monad_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_balance(address)?;
        let result = self.call_lens(data).await?;
        let balance = decode_get_lens_balance(&result)?;
        Ok(Some(AssetBalance::new_balance(
            AssetId::from_chain(Chain::Monad),
            primitives::Balance::stake_balance(balance.staked, balance.pending, Some(balance.rewards)),
        )))
    }

    fn map_lens_validator(validator: &MonadLensValidatorInfo, validator_names: &HashMap<u64, &str>, network_apy: f64) -> DelegationValidator {
        let validator_name = validator_names
            .get(&validator.validator_id)
            .map(|name| (*name).to_string())
            .unwrap_or_else(|| validator.validator_id.to_string());

        DelegationValidator::stake(
            Chain::Monad,
            validator.validator_id.to_string(),
            validator_name,
            validator.is_active,
            validator.commission.to_f64().unwrap_or(0.0) / MONAD_SCALE,
            if validator.apy_bps > 0 { validator.apy_bps as f64 / 100.0 } else { network_apy },
        )
    }

    fn map_lens_state(position: &MonadLensDelegation) -> DelegationState {
        match position.state {
            IMonadStakingLens::DelegationState::Active => DelegationState::Active,
            IMonadStakingLens::DelegationState::Activating => DelegationState::Activating,
            IMonadStakingLens::DelegationState::Deactivating => DelegationState::Deactivating,
            IMonadStakingLens::DelegationState::AwaitingWithdrawal => DelegationState::AwaitingWithdrawal,
            IMonadStakingLens::DelegationState::__Invalid => DelegationState::Inactive,
        }
    }

    async fn call_lens(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
        self.eth_call(STAKING_LENS_CONTRACT, &data).await
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use serde_json::json;

    use super::*;
    use crate::{method, testkit::TEST_MONAD_ADDRESS};

    #[tokio::test]
    async fn test_call_monad_delegations() {
        let rpc_client = mock_jsonrpc_client(|request_method, params| {
            assert_eq!(request_method, method::ETH_CALL);
            assert_eq!(
                params,
                &json!([
                    {
                        "data": "0x31cc13ba000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7",
                        "to": STAKING_LENS_CONTRACT
                    },
                    "latest"
                ])
            );
            Ok(json!("0x"))
        });
        let client = EthereumClient::new(rpc_client, primitives::EVMChain::Monad);

        assert_eq!(client.call_monad_delegations(TEST_MONAD_ADDRESS).await.unwrap(), Vec::<u8>::new());
    }
}
