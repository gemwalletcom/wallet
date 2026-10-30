use primitives::{SignerError, StakeType, chain_aptos::is_fungible_asset_token_id};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::models::TransactionPayload;
use crate::r#move::{EntryFunction, encode_argument, infer_type_tags, parse_function_id, parse_type_tag};
use crate::{APTOS_TRANSFER_FUNCTION, DELEGATION_POOL_ADD_STAKE_FUNCTION, DELEGATION_POOL_UNLOCK_FUNCTION, DELEGATION_POOL_WITHDRAW_FUNCTION, ENTRY_FUNCTION_PAYLOAD_TYPE, FUNGIBLE_TRANSFER_FUNCTION, OBJECT_CORE_TYPE};

#[derive(Clone, Debug, Deserialize)]
pub struct EntryFunctionPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<Value>,
}

impl EntryFunctionPayload {
    pub fn to_entry_function(&self, abi: Option<&[&str]>) -> Result<EntryFunction, SignerError> {
        if self.payload_type != ENTRY_FUNCTION_PAYLOAD_TYPE {
            return Err(SignerError::InvalidInput(format!("Unsupported Aptos payload type: {}", self.payload_type)));
        }

        let (module, function) = parse_function_id(&self.function)?;
        let ty_args = self.type_arguments.iter().map(|arg| parse_type_tag(arg)).collect::<Result<Vec<_>, _>>()?;

        let arg_types = match abi {
            Some(abi_types) => {
                if abi_types.len() != self.arguments.len() {
                    return Err(SignerError::InvalidInput("Aptos ABI length does not match arguments".to_string()));
                }
                abi_types.iter().map(|arg| parse_type_tag(arg)).collect::<Result<Vec<_>, _>>()?
            }
            None => infer_type_tags(&self.arguments)?,
        };

        let args = self.arguments.iter().zip(arg_types.iter()).map(|(value, arg_type)| encode_argument(value, arg_type)).collect::<Result<Vec<_>, _>>()?;

        Ok(EntryFunction { module, function, ty_args, args })
    }
}

impl From<EntryFunctionPayload> for TransactionPayload {
    fn from(payload: EntryFunctionPayload) -> Self {
        TransactionPayload {
            function: Some(payload.function),
            type_arguments: payload.type_arguments,
            arguments: payload.arguments,
            payload_type: payload.payload_type,
        }
    }
}

fn build_payload(function: &str, first_argument: &str, amount: &str) -> EntryFunctionPayload {
    EntryFunctionPayload {
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
        function: function.to_string(),
        type_arguments: vec![],
        arguments: vec![json!(first_argument), json!(amount)],
    }
}

pub fn build_delegation_pool_transaction_payload(stake_type: &StakeType, amount: &str) -> Option<EntryFunctionPayload> {
    match stake_type {
        StakeType::Stake(validator) => Some(build_payload(DELEGATION_POOL_ADD_STAKE_FUNCTION, &validator.id, amount)),
        StakeType::Unstake(delegation) => Some(build_payload(DELEGATION_POOL_UNLOCK_FUNCTION, &delegation.validator.id, amount)),
        StakeType::Withdraw(delegation) => Some(build_payload(DELEGATION_POOL_WITHDRAW_FUNCTION, &delegation.validator.id, amount)),
        StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => None,
    }
}

pub fn build_transfer_transaction_payload(recipient: &str, amount: &str) -> EntryFunctionPayload {
    build_payload(APTOS_TRANSFER_FUNCTION, recipient, amount)
}

pub fn build_token_transfer_transaction_payload(token_id: &str, recipient: &str, amount: &str) -> Result<EntryFunctionPayload, &'static str> {
    if !is_fungible_asset_token_id(token_id) {
        return Err("Invalid Aptos token ID format");
    }

    Ok(EntryFunctionPayload {
        payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
        function: FUNGIBLE_TRANSFER_FUNCTION.to_string(),
        type_arguments: vec![OBJECT_CORE_TYPE.to_string()],
        arguments: vec![json!(token_id), json!(recipient), json!(amount)],
    })
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, Delegation, DelegationValidator};

    use super::*;

    const TEST_POOL_ADDRESS: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e";

    fn to_json(payload: EntryFunctionPayload) -> Value {
        serde_json::to_value(TransactionPayload::from(payload)).unwrap()
    }

    #[test]
    fn test_build_delegation_pool_transaction_payload() {
        let pool = DelegationValidator::stake(Chain::Aptos, TEST_POOL_ADDRESS.to_string(), String::new(), true, 0.0, 0.0);

        assert_eq!(
            to_json(build_delegation_pool_transaction_payload(&StakeType::Stake(pool.clone()), "100000000").unwrap()),
            json!({
                "function": "0x1::delegation_pool::add_stake",
                "type_arguments": [],
                "arguments": [TEST_POOL_ADDRESS, "100000000"],
                "type": "entry_function_payload"
            })
        );
        assert_eq!(
            to_json(build_delegation_pool_transaction_payload(&StakeType::Unstake(Delegation::mock_with_validator(pool.clone())), "50000000").unwrap()),
            json!({
                "function": "0x1::delegation_pool::unlock",
                "type_arguments": [],
                "arguments": [TEST_POOL_ADDRESS, "50000000"],
                "type": "entry_function_payload"
            })
        );
        assert_eq!(
            to_json(build_delegation_pool_transaction_payload(&StakeType::Withdraw(Delegation::mock_with_validator(pool.clone())), "1102185008").unwrap()),
            json!({
                "function": "0x1::delegation_pool::withdraw",
                "type_arguments": [],
                "arguments": [TEST_POOL_ADDRESS, "1102185008"],
                "type": "entry_function_payload"
            })
        );
        assert_eq!(build_delegation_pool_transaction_payload(&StakeType::Rewards(vec![pool]), "1").map(to_json), None);
    }

    #[test]
    fn test_build_token_transfer_transaction_payload_fungible_asset() {
        let result = to_json(build_token_transfer_transaction_payload("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b", TEST_POOL_ADDRESS, "1").unwrap());
        let expected = json!({
            "function": "0x1::primary_fungible_store::transfer",
            "type_arguments": ["0x1::object::ObjectCore"],
            "arguments": ["0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b", TEST_POOL_ADDRESS, "1"],
            "type": "entry_function_payload"
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_token_transfer_transaction_payload_invalid() {
        let err = build_token_transfer_transaction_payload("invalid", TEST_POOL_ADDRESS, "1").unwrap_err();
        assert_eq!(err, "Invalid Aptos token ID format");
    }
}
