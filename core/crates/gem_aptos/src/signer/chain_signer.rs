use hex::encode;
use primitives::{ChainSigner, SignerError, SignerInput, TransactionInputType, TransactionLoadMetadata};
use serde_json::{Value, from_str};
use std::str::from_utf8;

use super::abi::{HYPERION_PARTNER_SWAP_PARAMS, PANORA_ROUTER_ENTRY_PARAMS, PANORA_ROUTER_FUNCTION, PANORA_ROUTER_MODULE};
use super::{
    EntryFunction, EntryFunctionPayload, build_raw_transaction, build_submit_transaction_bcs, build_token_transfer_transaction_payload, build_transfer_transaction_payload, expiration_timestamp_secs, sign_message as sign_aptos_message,
    sign_raw_transaction,
};
use crate::AccountAddress;
use crate::{DELEGATION_POOL_ADD_STAKE_FUNCTION, DELEGATION_POOL_UNLOCK_FUNCTION, DELEGATION_POOL_WITHDRAW_FUNCTION, HYPERION_PARTNER_SWAP_FUNCTION};

const APTOS_CHAIN_ID: u8 = 1;

const STAKE_ENTRY_PARAMS: [&str; 2] = ["address", "u64"];
const TRANSFER_ENTRY_PARAMS: [&str; 2] = ["address", "u64"];
const FUNGIBLE_TRANSFER_ENTRY_PARAMS: [&str; 3] = ["address", "address", "u64"];

#[derive(Default)]
pub struct AptosChainSigner;

impl ChainSigner for AptosChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let payload = build_transfer_transaction_payload(&input.destination_address, &input.value.to_string());

        let gas_limit = input.fee.gas_limit()?;
        self.sign_payload(payload, Some(&TRANSFER_ENTRY_PARAMS), input, private_key, gas_limit)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let gas_limit = input.fee.gas_limit()?;
        let (payload, abi) = token_transfer_payload(input)?;
        self.sign_payload(payload, Some(abi), input, private_key, gas_limit)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = match &input.input_type {
            TransactionInputType::Swap { swap_data: data, .. } => data,
            _ => return Err(SignerError::InvalidInput("Expected Aptos swap input".to_string())),
        };

        let payload: EntryFunctionPayload = from_str(swap_data.data.data.as_str())?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        let signed = self.sign_entry_function(entry_function, input, private_key, input.fee.gas_limit()?)?;
        Ok(vec![signed])
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let data = match &input.metadata {
            TransactionLoadMetadata::Aptos { data: Some(data), .. } => data,
            _ => return Err(SignerError::InvalidInput("Missing Aptos stake payload".to_string())),
        };

        let payload: EntryFunctionPayload = from_str(data)?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        let signed = self.sign_entry_function(entry_function, input, private_key, input.fee.gas_limit()?)?;
        Ok(vec![signed])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let (signature, _) = sign_aptos_message(message, private_key)?;
        Ok(format!("0x{}", encode(signature)))
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let (payload, gas_limit) = get_generic_payload(input)?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        self.sign_entry_function(entry_function, input, private_key, gas_limit)
    }
}

impl AptosChainSigner {
    fn sign_payload(&self, payload: EntryFunctionPayload, abi: Option<&[&str]>, input: &SignerInput, private_key: &[u8], gas_limit: u64) -> Result<String, SignerError> {
        let entry_function = payload.to_entry_function(abi)?;
        self.sign_entry_function(entry_function, input, private_key, gas_limit)
    }

    fn sign_entry_function(&self, entry_function: EntryFunction, input: &SignerInput, private_key: &[u8], gas_limit: u64) -> Result<String, SignerError> {
        let sender = AccountAddress::from_hex(&input.sender_address)?;
        let sequence = sequence_from_metadata(&input.metadata)?;
        let gas_unit_price = input.fee.gas_price_u64()?;
        let expiration = expiration_timestamp_secs()?;

        let raw_tx = build_raw_transaction(sender, sequence, entry_function, gas_limit, gas_unit_price, expiration, APTOS_CHAIN_ID);
        let (signature, public_key) = sign_raw_transaction(&raw_tx, private_key)?;

        build_submit_transaction_bcs(raw_tx, signature, public_key)
    }
}

fn get_payload_abi(payload: &EntryFunctionPayload) -> Option<&'static [&'static str]> {
    match payload.function.as_str() {
        DELEGATION_POOL_ADD_STAKE_FUNCTION | DELEGATION_POOL_UNLOCK_FUNCTION | DELEGATION_POOL_WITHDRAW_FUNCTION => Some(&STAKE_ENTRY_PARAMS),
        HYPERION_PARTNER_SWAP_FUNCTION => Some(&HYPERION_PARTNER_SWAP_PARAMS),
        _ => None,
    }
}

fn prepare_payload(payload: EntryFunctionPayload) -> Result<(EntryFunctionPayload, Option<&'static [&'static str]>), SignerError> {
    if is_panora_router(&payload.function) {
        return prepare_panora_payload(payload);
    }

    let abi = get_payload_abi(&payload);
    Ok((payload, abi))
}

fn prepare_panora_payload(mut payload: EntryFunctionPayload) -> Result<(EntryFunctionPayload, Option<&'static [&'static str]>), SignerError> {
    let expected = PANORA_ROUTER_ENTRY_PARAMS.len();
    let before_len = payload.arguments.len();
    if before_len + 1 == expected {
        payload.arguments.insert(0, Value::Null);
    }
    if payload.arguments.len() != expected {
        return Err(SignerError::InvalidInput("Aptos ABI length does not match arguments".to_string()));
    }

    Ok((payload, Some(&PANORA_ROUTER_ENTRY_PARAMS)))
}

fn get_generic_payload(input: &SignerInput) -> Result<(EntryFunctionPayload, u64), SignerError> {
    let data = match &input.input_type {
        TransactionInputType::Generic { extra, .. } | TransactionInputType::Payment { extra, .. } => extra.data.as_ref(),
        _ => return Err(SignerError::InvalidInput("Expected Aptos generic input".to_string())),
    };

    let json = if let Some(bytes) = data {
        if bytes.is_empty() {
            return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
        }
        from_utf8(bytes).map_err(|_| SignerError::InvalidInput("Aptos payload must be valid UTF-8".to_string()))?.to_string()
    } else if let TransactionLoadMetadata::Aptos { data: Some(json), .. } = &input.metadata {
        json.clone()
    } else {
        return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
    };

    let payload: EntryFunctionPayload = from_str(&json)?;
    Ok((payload, input.fee.gas_limit()?))
}

fn is_panora_router(function_id: &str) -> bool {
    let mut parts = function_id.split("::");
    let _address = parts.next();
    let module = parts.next();
    let function = parts.next();

    module == Some(PANORA_ROUTER_MODULE) && function == Some(PANORA_ROUTER_FUNCTION)
}

fn sequence_from_metadata(metadata: &TransactionLoadMetadata) -> Result<u64, SignerError> {
    match metadata {
        TransactionLoadMetadata::Aptos { sequence, .. } => Ok(*sequence),
        _ => Err(SignerError::InvalidInput("Missing Aptos sequence".to_string())),
    }
}

fn token_transfer_payload(input: &SignerInput) -> Result<(EntryFunctionPayload, &'static [&'static str]), SignerError> {
    let asset = input.input_type.get_asset();
    let token_id = asset.token_id().ok_or_else(|| SignerError::invalid_input("Missing Aptos token id"))?;
    let payload = build_token_transfer_transaction_payload(token_id, &input.destination_address, &input.value.to_string()).map_err(SignerError::invalid_input)?;

    Ok((payload, &FUNGIBLE_TRANSFER_ENTRY_PARAMS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FUNGIBLE_TRANSFER_FUNCTION, OBJECT_CORE_TYPE};
    use primitives::{AssetId, Chain, GasPriceType, SignerInput, TransactionFee, TransactionLoadInput};

    #[test]
    fn gas_limit_uses_fee_model() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b");
        let input = SignerInput::new(
            input,
            TransactionFee::new_gas_price_type(GasPriceType::regular(1u64), 42u64.into(), 42u64.into(), Default::default(), AssetId::from_chain(Chain::Aptos)),
        );

        assert_eq!(input.fee.gas_limit().unwrap(), 42);
    }

    #[test]
    fn test_prepare_payload_hyperion_partner_swap() {
        let payload: EntryFunctionPayload = from_str(&format!(
            r#"{{"type":"entry_function_payload","function":"{HYPERION_PARTNER_SWAP_FUNCTION}","type_arguments":[],"arguments":[["0x925660b8618394809f89f8002e2926600c775221f43bf1919782b297a79400d8"],"0xa","0xbae207659db88bea0cbead6da0ed00aac12edcdda169e591cd41c94180b46f3b","100000000","800000","gemwallet","500000","0xc09d385527743bb03ed7847bb9180b5ff2263d38d5a93f1c9b3068f8505f6488"]}}"#
        ))
        .unwrap();
        let (payload, abi) = prepare_payload(payload).unwrap();
        let entry_function = payload.to_entry_function(abi).unwrap();

        assert_eq!(entry_function.module.address, AccountAddress::from_hex("0x8b4a2c4bb53857c718a04c020b98f8c2e1f99a68b0f57389a8bf5434cd22e05c").unwrap());
        assert_eq!(entry_function.module.name, "partnership");
        assert_eq!(entry_function.function, "partner_swap");
        assert_eq!(
            entry_function.args.iter().map(hex::encode).collect::<Vec<_>>(),
            vec![
                "01925660b8618394809f89f8002e2926600c775221f43bf1919782b297a79400d8",
                "000000000000000000000000000000000000000000000000000000000000000a",
                "bae207659db88bea0cbead6da0ed00aac12edcdda169e591cd41c94180b46f3b",
                "00e1f50500000000",
                "00350c0000000000",
                "0967656d77616c6c6574",
                "20a1070000000000",
                "c09d385527743bb03ed7847bb9180b5ff2263d38d5a93f1c9b3068f8505f6488",
            ]
        );
    }

    #[test]
    fn token_transfer_payload_uses_fungible_asset_transfer() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b");
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);
        let (payload, abi) = token_transfer_payload(&input).unwrap();

        assert_eq!(payload.function, FUNGIBLE_TRANSFER_FUNCTION);
        assert_eq!(payload.type_arguments, vec![OBJECT_CORE_TYPE.to_string()]);
        assert_eq!(abi, &FUNGIBLE_TRANSFER_ENTRY_PARAMS);
    }

    #[test]
    fn token_transfer_payload_rejects_invalid_token_id() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("invalid");
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);
        let err = token_transfer_payload(&input).unwrap_err();

        match err {
            SignerError::InvalidInput(message) => assert_eq!(message, "Invalid Aptos token ID format"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
