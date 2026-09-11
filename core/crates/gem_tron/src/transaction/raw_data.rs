use gem_encoding::protobuf::MessageEncode;
use num_bigint::BigUint;
use primitives::{Address as _, ApprovalData, SignerError};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::hex_bytes;

use super::{TronContract, protobuf};
use crate::models::TronContractType;
use crate::trc20;

#[derive(Deserialize)]
pub(crate) struct RawDataJson {
    contract: Vec<RawContractJson>,
    expiration: u64,
    #[serde(with = "hex_bytes")]
    ref_block_bytes: Vec<u8>,
    #[serde(with = "hex_bytes")]
    ref_block_hash: Vec<u8>,
    timestamp: u64,
    fee_limit: Option<u64>,
    #[serde(default, with = "hex_bytes::option")]
    data: Option<Vec<u8>>,
}

impl RawDataJson {
    pub(crate) fn approval(&self) -> Result<Option<ApprovalData>, SignerError> {
        let [contract] = self.contract.as_slice() else {
            return Ok(None);
        };
        match TronContract::from_json_value(contract.contract_type, contract.parameter.value.clone())? {
            TronContract::TriggerSmart {
                contract,
                data,
                call_value,
                call_token_value,
                ..
            } => {
                if !data.starts_with(&trc20::TRC20_APPROVE_SELECTOR) {
                    return Ok(None);
                }
                let approval = trc20::decode_approval(&data).ok_or_else(|| SignerError::invalid_input("Invalid TRC20 approval calldata"))?;
                if call_value.is_some_and(|value| value != 0) || call_token_value.is_some_and(|value| value != 0) {
                    return SignerError::invalid_input_err("TRC20 approval must not transfer native tokens");
                }
                Ok(Some(ApprovalData {
                    token: contract.encode(),
                    spender: approval.spender.encode(),
                    is_unlimited: approval.value == BigUint::from_bytes_be(&[0xff; 32]),
                    value: approval.value,
                }))
            }
            TronContract::Transfer { .. }
            | TronContract::VoteWitness { .. }
            | TronContract::FreezeBalanceV2 { .. }
            | TronContract::UnfreezeBalanceV2 { .. }
            | TronContract::DelegateResource { .. }
            | TronContract::UnDelegateResource { .. }
            | TronContract::WithdrawBalance { .. }
            | TronContract::WithdrawExpireUnfreeze { .. } => Ok(None),
        }
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, SignerError> {
        let contracts = self
            .contract
            .iter()
            .map(|contract| TronContract::from_json_value(contract.contract_type, contract.parameter.value.clone()).map(|contract| protobuf::ContractEnvelope::from(&contract)))
            .collect::<Result<Vec<_>, SignerError>>()?;

        Ok(protobuf::RawData {
            ref_block_bytes: Some(self.ref_block_bytes.clone()),
            ref_block_hash: Some(self.ref_block_hash.clone()),
            expiration: (self.expiration > 0).then_some(self.expiration),
            data: self.data.clone(),
            contracts,
            timestamp: (self.timestamp > 0).then_some(self.timestamp),
            fee_limit: self.fee_limit.filter(|value| *value > 0),
        }
        .encode())
    }
}

#[derive(Deserialize)]
struct RawContractJson {
    #[serde(rename = "type")]
    contract_type: TronContractType,
    parameter: RawParameterJson,
}

#[derive(Deserialize)]
struct RawParameterJson {
    value: Value,
}
