use gem_evm::eip712::{EIP712Field, EIP712TypedValue, eip712_domain_types, parse_eip712_json, validate_eip712_chain_id};
use num_bigint::BigUint;
use primitives::Chain;
use serde_json::Value;
use serde_serializers::biguint_from_hex_str;

use crate::error::PaymentError;
use crate::wallet_connect_pay::model::TypedDataTransfer;

const TYPE_EIP712_DOMAIN: &str = "EIP712Domain";
const PRIMARY_TYPE_PERMIT_TRANSFER_FROM: &str = "PermitTransferFrom";
const PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION: &str = "TransferWithAuthorization";
const PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION: &str = "ReceiveWithAuthorization";

pub(super) fn map_typed_data(chain: Chain, value: &Value) -> Result<TypedDataTransfer, PaymentError> {
    let typed_data = match value {
        Value::String(json) => serde_json::from_str(json).map_err(|error| PaymentError::invalid_request(format!("Invalid payment signature: {error}")))?,
        value => value.clone(),
    };
    let typed_data = with_domain_schema(typed_data)?;
    let chain_id = chain
        .network_id_value()
        .ok_or_else(|| PaymentError::invalid_request(format!("{} has no chain id", chain.as_ref())))?;
    validate_eip712_chain_id(&typed_data.to_string(), chain_id).map_err(PaymentError::invalid_request)?;
    let message = parse_eip712_json(&typed_data).map_err(PaymentError::invalid_request)?;
    if message.domain.chain_id.is_none() {
        return Err(PaymentError::invalid_request("Payment signature has no chain id"));
    }

    let (token, amount, from, recipient) = match message.primary_type.as_str() {
        PRIMARY_TYPE_PERMIT_TRANSFER_FROM => {
            let permitted = get_fields(&message.message, "permitted")?;
            (
                get_address(permitted, "token")?,
                get_amount(permitted, "amount")?,
                None,
                get_address(&message.message, "spender")?,
            )
        }
        PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION | PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION => (
            message
                .domain
                .verifying_contract
                .clone()
                .ok_or_else(|| PaymentError::invalid_request("Payment signature has no verifying contract"))?,
            get_amount(&message.message, "value")?,
            Some(get_address(&message.message, "from")?),
            get_address(&message.message, "to")?,
        ),
        primary_type => return Err(PaymentError::invalid_request(format!("Unsupported payment signature: {primary_type}"))),
    };

    Ok(TypedDataTransfer {
        token,
        amount,
        from,
        recipient,
        typed_data: typed_data.to_string(),
    })
}

fn with_domain_schema(typed_data: Value) -> Result<Value, PaymentError> {
    let Value::Object(mut typed_data) = typed_data else {
        return Err(PaymentError::invalid_request("Expected typed data object"));
    };
    let Some(Value::Object(domain)) = typed_data.get("domain").cloned() else {
        return Err(PaymentError::invalid_request("Payment signature has no domain"));
    };
    let Some(Value::Object(types)) = typed_data.get_mut("types") else {
        return Err(PaymentError::invalid_request("Payment signature has no types"));
    };
    if types.contains_key(TYPE_EIP712_DOMAIN) {
        return Ok(Value::Object(typed_data));
    }
    let fields = eip712_domain_types();
    for (name, value) in &domain {
        if !fields.iter().any(|field| field.name == *name) {
            return Err(PaymentError::invalid_request(format!("Unsupported EIP712 domain field: {name}")));
        }
        if value.is_null() {
            return Err(PaymentError::invalid_request(format!("Missing EIP712 domain field value: {name}")));
        }
    }
    let schema: Vec<_> = fields.into_iter().filter(|field| domain.contains_key(&field.name)).collect();
    let schema = serde_json::to_value(schema).map_err(|error| PaymentError::invalid_request(error.to_string()))?;
    types.insert(TYPE_EIP712_DOMAIN.to_string(), schema);
    Ok(Value::Object(typed_data))
}

fn get_fields<'a>(fields: &'a [EIP712Field], name: &str) -> Result<&'a [EIP712Field], PaymentError> {
    match get_field(fields, name)? {
        EIP712TypedValue::Struct { fields } => Ok(fields),
        _ => Err(PaymentError::invalid_request(format!("Payment signature {name} is not a struct"))),
    }
}

fn get_address(fields: &[EIP712Field], name: &str) -> Result<String, PaymentError> {
    match get_field(fields, name)? {
        EIP712TypedValue::Address { value } => Ok(value.clone()),
        _ => Err(PaymentError::invalid_request(format!("Payment signature {name} is not an address"))),
    }
}

fn get_amount(fields: &[EIP712Field], name: &str) -> Result<BigUint, PaymentError> {
    let EIP712TypedValue::Uint256 { value } = get_field(fields, name)? else {
        return Err(PaymentError::invalid_request(format!("Payment signature {name} is not an amount")));
    };
    match value.starts_with("0x") {
        true => biguint_from_hex_str(value).ok(),
        false => value.parse().ok(),
    }
    .ok_or_else(|| PaymentError::invalid_request(format!("Invalid payment signature {name}: {value}")))
}

fn get_field<'a>(fields: &'a [EIP712Field], name: &str) -> Result<&'a EIP712TypedValue, PaymentError> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
        .ok_or_else(|| PaymentError::invalid_request(format!("Payment signature has no {name}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::testkit::{
        PERMIT_TRANSFER_FROM, PYUSD_TOKEN_ID, RECEIVE_WITH_AUTHORIZATION, TEST_ACCOUNT_WITHOUT_ALLOWANCE, TEST_AUTHORIZATION_RECIPIENT, TEST_PERMIT_SPENDER,
    };
    use primitives::asset_constants::ETHEREUM_USDT_TOKEN_ID;

    fn permit_transfer_from() -> Value {
        serde_json::from_str(PERMIT_TRANSFER_FROM).unwrap()
    }

    fn receive_with_authorization() -> Value {
        serde_json::from_str(RECEIVE_WITH_AUTHORIZATION).unwrap()
    }

    fn without_declared_domain(mut typed_data: Value) -> Value {
        typed_data["types"].as_object_mut().unwrap().remove(TYPE_EIP712_DOMAIN);
        typed_data
    }

    #[test]
    fn test_map_typed_data() {
        let permit = permit_transfer_from();
        assert_eq!(
            map_typed_data(Chain::Ethereum, &Value::String(permit.to_string())),
            Ok(TypedDataTransfer {
                token: ETHEREUM_USDT_TOKEN_ID.to_string(),
                amount: BigUint::from(100_000u32),
                from: None,
                recipient: TEST_PERMIT_SPENDER.to_string(),
                typed_data: permit.to_string(),
            })
        );

        let authorization = receive_with_authorization();
        assert_eq!(
            map_typed_data(Chain::Ethereum, &authorization),
            Ok(TypedDataTransfer {
                token: PYUSD_TOKEN_ID.to_string(),
                amount: BigUint::from(100_000u32),
                from: Some(TEST_ACCOUNT_WITHOUT_ALLOWANCE.to_string()),
                recipient: TEST_AUTHORIZATION_RECIPIENT.to_string(),
                typed_data: authorization.to_string(),
            })
        );

        let mut transfer = authorization.clone();
        transfer["primaryType"] = Value::from(PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION);
        transfer["types"][PRIMARY_TYPE_TRANSFER_WITH_AUTHORIZATION] = transfer["types"][PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION].take();
        transfer["types"].as_object_mut().unwrap().remove(PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION);
        assert_eq!(
            map_typed_data(Chain::Ethereum, &transfer).map(|transfer| transfer.recipient),
            Ok(TEST_AUTHORIZATION_RECIPIENT.to_string())
        );

        let undeclared = without_declared_domain(permit.clone());
        let signed: Value = serde_json::from_str(&map_typed_data(Chain::Ethereum, &undeclared).unwrap().typed_data).unwrap();
        assert_eq!(signed["types"][TYPE_EIP712_DOMAIN], permit["types"][TYPE_EIP712_DOMAIN], "a domain left undeclared is declared from its fields");

        assert_eq!(
            map_typed_data(Chain::Polygon, &permit),
            Err(PaymentError::invalid_request("Chain ID mismatch: expected 137, got 1")),
            "the domain chain must be the quote chain"
        );
        let mut unknown = permit.clone();
        unknown["primaryType"] = Value::from("TokenPermissions");
        unknown["message"] = permit["message"]["permitted"].clone();
        assert_eq!(
            map_typed_data(Chain::Ethereum, &unknown),
            Err(PaymentError::invalid_request("Unsupported payment signature: TokenPermissions"))
        );
        let mut salted = without_declared_domain(permit.clone());
        salted["domain"]["extra"] = Value::from("1");
        assert_eq!(
            map_typed_data(Chain::Ethereum, &salted),
            Err(PaymentError::invalid_request("Unsupported EIP712 domain field: extra")),
            "a domain field the schema cannot declare is not signed"
        );
        let mut unsigned_value = authorization.clone();
        unsigned_value["types"][PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION] = authorization["types"][PRIMARY_TYPE_RECEIVE_WITH_AUTHORIZATION]
            .as_array()
            .unwrap()
            .iter()
            .filter(|field| field["name"] != "value")
            .cloned()
            .collect();
        assert_eq!(
            map_typed_data(Chain::Ethereum, &unsigned_value),
            Err(PaymentError::invalid_request("Payment signature has no value")),
            "a value the schema does not declare is not part of the signature"
        );
        let mut unbound = without_declared_domain(permit);
        unbound["domain"].as_object_mut().unwrap().remove("chainId");
        assert_eq!(
            map_typed_data(Chain::Ethereum, &unbound),
            Err(PaymentError::invalid_request("Payment signature has no chain id")),
            "a signature without a chain replays on every chain"
        );
    }
}
