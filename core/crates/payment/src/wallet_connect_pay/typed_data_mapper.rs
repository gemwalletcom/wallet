use gem_evm::eip712::{EIP712Field, find_field_string, find_field_struct, parse_eip712_json};
use num_bigint::BigUint;
use serde_json::Value;
use serde_serializers::biguint_from_hex_str;

use crate::error::PaymentError;
use crate::wallet_connect_pay::model::TypedDataTransfer;

const PRIMARY_TYPE_PERMIT_TRANSFER_FROM: &str = "PermitTransferFrom";
const PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION: &str = "TransferWithAuthorization";
const PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION: &str = "ReceiveWithAuthorization";

pub(super) fn map_typed_data(typed_data: &str) -> Result<TypedDataTransfer, PaymentError> {
    let value: Value = serde_json::from_str(typed_data).map_err(|error| PaymentError::invalid_request(format!("Invalid payment signature: {error}")))?;
    let message = parse_eip712_json(&value).map_err(PaymentError::invalid_request)?;
    if message.domain.chain_id.is_none() {
        return Err(PaymentError::invalid_request("Payment signature has no chain id"));
    }
    let verifying_contract = message.domain.verifying_contract.clone().ok_or_else(|| missing("verifying contract"))?;
    let (token, amount, from, recipient) = match message.primary_type.as_str() {
        PRIMARY_TYPE_PERMIT_TRANSFER_FROM => {
            let permitted = find_field_struct(&message.message, "permitted").ok_or_else(|| missing("permitted"))?;
            (get_field(permitted, "token")?, get_amount(permitted, "amount")?, None, get_field(&message.message, "spender")?)
        }
        PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION | PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION => (
            verifying_contract.clone(),
            get_amount(&message.message, "value")?,
            Some(get_field(&message.message, "from")?),
            get_field(&message.message, "to")?,
        ),
        primary_type => return Err(PaymentError::invalid_request(format!("Unsupported payment signature: {primary_type}"))),
    };
    Ok(TypedDataTransfer {
        token,
        amount,
        from,
        recipient,
        verifying_contract,
        typed_data: typed_data.to_string(),
    })
}

fn get_field(fields: &[EIP712Field], name: &str) -> Result<String, PaymentError> {
    find_field_string(fields, name).ok_or_else(|| missing(name))
}

fn get_amount(fields: &[EIP712Field], name: &str) -> Result<BigUint, PaymentError> {
    let value = get_field(fields, name)?;
    match value.starts_with("0x") {
        true => biguint_from_hex_str(&value).ok(),
        false => value.parse().ok(),
    }
    .ok_or_else(|| PaymentError::invalid_request(format!("Invalid payment signature {name}: {value}")))
}

fn missing(name: &str) -> PaymentError {
    PaymentError::invalid_request(format!("Payment signature has no {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::testkit::{
        PERMIT_TRANSFER_FROM, PYUSD_TOKEN_ID, RECEIVE_WITH_AUTHORIZATION, TEST_ACCOUNT_WITHOUT_ALLOWANCE, TEST_AUTHORIZATION_RECIPIENT, TEST_PERMIT_SPENDER,
    };
    use primitives::asset_constants::ETHEREUM_USDT_TOKEN_ID;
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;

    fn permit_transfer_from() -> Value {
        serde_json::from_str(PERMIT_TRANSFER_FROM).unwrap()
    }

    #[test]
    fn test_map_typed_data() {
        assert_eq!(
            map_typed_data(PERMIT_TRANSFER_FROM),
            Ok(TypedDataTransfer {
                token: ETHEREUM_USDT_TOKEN_ID.to_string(),
                amount: BigUint::from(100_000u32),
                from: None,
                recipient: TEST_PERMIT_SPENDER.to_string(),
                verifying_contract: UNISWAP_PERMIT2_CONTRACT.to_string(),
                typed_data: PERMIT_TRANSFER_FROM.to_string(),
            })
        );
        assert_eq!(
            map_typed_data(RECEIVE_WITH_AUTHORIZATION),
            Ok(TypedDataTransfer {
                token: PYUSD_TOKEN_ID.to_string(),
                amount: BigUint::from(100_000u32),
                from: Some(TEST_ACCOUNT_WITHOUT_ALLOWANCE.to_string()),
                recipient: TEST_AUTHORIZATION_RECIPIENT.to_string(),
                verifying_contract: PYUSD_TOKEN_ID.to_string(),
                typed_data: RECEIVE_WITH_AUTHORIZATION.to_string(),
            })
        );
        assert_eq!(
            map_typed_data(&RECEIVE_WITH_AUTHORIZATION.replace(PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION, PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION))
                .map(|transfer| transfer.recipient),
            Ok(TEST_AUTHORIZATION_RECIPIENT.to_string())
        );

        let mut unknown = permit_transfer_from();
        unknown["primaryType"] = Value::from("TokenPermissions");
        unknown["message"] = unknown["message"]["permitted"].take();
        assert_eq!(
            map_typed_data(&unknown.to_string()),
            Err(PaymentError::invalid_request("Unsupported payment signature: TokenPermissions"))
        );
        let mut unsigned_value = permit_transfer_from();
        unsigned_value["types"]["TokenPermissions"] = serde_json::json!([{"name": "token", "type": "address"}]);
        assert_eq!(
            map_typed_data(&unsigned_value.to_string()),
            Err(PaymentError::invalid_request("Payment signature has no amount")),
            "an amount the schema does not declare is not part of the signature"
        );
        let mut unbound = permit_transfer_from();
        unbound["domain"].as_object_mut().unwrap().remove("chainId");
        unbound["types"]["EIP712Domain"] = serde_json::json!([{"name": "name", "type": "string"}, {"name": "verifyingContract", "type": "address"}]);
        assert_eq!(
            map_typed_data(&unbound.to_string()),
            Err(PaymentError::invalid_request("Payment signature has no chain id")),
            "a signature without a chain replays on every chain"
        );
    }
}
