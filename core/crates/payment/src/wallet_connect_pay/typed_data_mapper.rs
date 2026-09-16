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
    use crate::wallet_connect_pay::testkit::{TRANSFER_WITH_AUTHORIZATION, mock_permit_transfer_from};
    use primitives::asset_constants::{BASE_USDC_TOKEN_ID, POLYGON_USDT_TOKEN_ID};
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};

    fn permit_transfer_from() -> Value {
        mock_permit_transfer_from("1000000")
    }

    fn transfer_with_authorization() -> Value {
        serde_json::from_str(TRANSFER_WITH_AUTHORIZATION).unwrap()
    }

    #[test]
    fn test_map_typed_data_reads_a_permit2_transfer_and_declares_the_domain() {
        let transfer = map_typed_data(Chain::Polygon, &Value::String(permit_transfer_from().to_string())).unwrap();

        assert!(transfer.token.eq_ignore_ascii_case(POLYGON_USDT_TOKEN_ID));
        assert_eq!(transfer.amount, BigUint::from(1_000_000u32));
        assert_eq!(transfer.from, None);
        assert!(transfer.recipient.eq_ignore_ascii_case(TEST_EVM_RECIPIENT));

        let signed: Value = serde_json::from_str(&transfer.typed_data).unwrap();
        assert_eq!(
            signed["types"][TYPE_EIP712_DOMAIN],
            serde_json::json!([
                {"name": "name", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"},
            ])
        );
    }

    #[test]
    fn test_map_typed_data_reads_a_transfer_with_authorization() {
        let transfer = map_typed_data(Chain::Base, &transfer_with_authorization()).unwrap();

        assert!(transfer.token.eq_ignore_ascii_case(BASE_USDC_TOKEN_ID));
        assert_eq!(transfer.amount, BigUint::from(250_000u32));
        assert!(transfer.from.as_deref().is_some_and(|from| from.eq_ignore_ascii_case(TEST_EVM_SENDER)));
        assert!(transfer.recipient.eq_ignore_ascii_case(TEST_EVM_RECIPIENT));
    }

    #[test]
    fn test_map_typed_data_refuses_what_it_cannot_vouch_for() {
        assert!(map_typed_data(Chain::Ethereum, &permit_transfer_from()).is_err(), "domain chain differs from the quote chain");

        let mut unknown = permit_transfer_from();
        unknown["primaryType"] = Value::String("TokenPermissions".to_string());
        unknown["message"] = serde_json::json!({"token": POLYGON_USDT_TOKEN_ID, "amount": "1000000"});
        assert_eq!(
            map_typed_data(Chain::Polygon, &unknown),
            Err(PaymentError::invalid_request("Unsupported payment signature: TokenPermissions"))
        );

        let mut salted = permit_transfer_from();
        salted["domain"]["extra"] = Value::String("1".to_string());
        assert!(map_typed_data(Chain::Polygon, &salted).is_err(), "a domain field the schema cannot declare is not signed");

        let mut unsigned_value = transfer_with_authorization();
        unsigned_value["types"]["TransferWithAuthorization"] = serde_json::json!([{"name": "from", "type": "address"}, {"name": "to", "type": "address"}]);
        assert_eq!(
            map_typed_data(Chain::Base, &unsigned_value),
            Err(PaymentError::invalid_request("Payment signature has no value")),
            "a value the schema does not declare is not part of the signature"
        );

        let mut declared = permit_transfer_from();
        declared["types"][TYPE_EIP712_DOMAIN] = serde_json::json!([{"name": "name", "type": "string"}, {"name": "chainId", "type": "uint256"}, {"name": "verifyingContract", "type": "address"}]);
        let transfer = map_typed_data(Chain::Polygon, &declared).unwrap();
        let signed: Value = serde_json::from_str(&transfer.typed_data).unwrap();
        assert_eq!(signed["types"][TYPE_EIP712_DOMAIN].as_array().unwrap().len(), 3);
    }
}
