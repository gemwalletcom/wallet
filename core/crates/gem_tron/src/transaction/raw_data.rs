use gem_encoding::protobuf::MessageEncode;
use primitives::{Address as _, SignerError, hex};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::hex_bytes;

use super::{TransactionApproval, TronContract, WalletConnectContract, WalletConnectToken, protobuf};
use crate::models::{TriggerSmartContractData, TronContractType};

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
    pub(crate) fn single_contract(&self) -> Result<Option<TronContract>, SignerError> {
        let [contract] = self.contract.as_slice() else {
            return Ok(None);
        };
        TronContract::from_json_value(contract.contract_type, contract.parameter.value.clone()).map(Some)
    }

    pub(crate) fn trigger_smart_contract_data(&self) -> Result<Option<TriggerSmartContractData>, SignerError> {
        let Some(TronContract::TriggerSmart { owner, contract, data, call_value, .. }) = self.single_contract()? else {
            return Ok(None);
        };
        Ok(Some(TriggerSmartContractData {
            contract_address: contract.encode(),
            data: hex::encode(data),
            owner_address: owner.encode(),
            fee_limit: self.fee_limit,
            call_value,
        }))
    }

    pub(crate) fn wallet_connect_contract(&self) -> Result<Option<WalletConnectContract>, SignerError> {
        let Some(contract) = self.single_contract()? else {
            return Ok(None);
        };
        match contract {
            TronContract::Transfer { to, amount, .. } => Ok(Some(WalletConnectContract {
                to: to.encode(),
                value: amount,
                token: None,
                approval: None,
            })),
            TronContract::TriggerSmart {
                contract,
                data,
                call_value,
                call_token_value,
                token_id,
                ..
            } => {
                let to = contract.encode();
                let approval = TransactionApproval::decode(contract, &data)?;
                if approval.is_some() && (call_value.is_some_and(|value| value != 0) || call_token_value.is_some_and(|value| value != 0)) {
                    return SignerError::invalid_input_err("Approval must not transfer native tokens");
                }
                Ok(Some(WalletConnectContract {
                    to,
                    value: call_value.unwrap_or_default(),
                    token: call_token_value.filter(|value| *value != 0).map(|value| WalletConnectToken { id: token_id.unwrap_or_default(), value }),
                    approval,
                }))
            }
            TronContract::DelegateResource { receiver, .. } | TronContract::UnDelegateResource { receiver, .. } => Ok(Some(WalletConnectContract {
                to: receiver.encode(),
                value: 0,
                token: None,
                approval: None,
            })),
            TronContract::VoteWitness { .. } | TronContract::FreezeBalanceV2 { .. } | TronContract::UnfreezeBalanceV2 { .. } | TronContract::WithdrawBalance { .. } | TronContract::WithdrawExpireUnfreeze { .. } => Ok(None),
        }
    }

    pub(crate) fn owners(&self) -> Result<Vec<String>, SignerError> {
        self.contract
            .iter()
            .map(|contract| TronContract::from_json_value(contract.contract_type, contract.parameter.value.clone()).map(|contract| contract.owner().encode()))
            .collect()
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
