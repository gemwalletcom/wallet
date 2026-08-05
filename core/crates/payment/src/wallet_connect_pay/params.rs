use gem_evm::eip712::{EIP712Type, eip712_domain_types};
use primitives::ValueAccess;
use serde_json::{Map, Value};
use std::fmt::Display;

use crate::error::PaymentError;

const SOLANA_METHOD_PREFIX: &str = "solana_";
const METHOD_ETH_SIGN_TYPED_DATA_V4: &str = "eth_signTypedData_v4";
const TYPE_EIP712_DOMAIN: &str = "EIP712Domain";

pub fn map_signer_params(method: &str, params: &Value) -> Result<Value, PaymentError> {
    if method.starts_with(SOLANA_METHOD_PREFIX) {
        return Ok(unwrapped_solana_transaction(params));
    }
    if method == METHOD_ETH_SIGN_TYPED_DATA_V4 {
        return typed_data_with_schema(params);
    }
    Ok(params.clone())
}

fn unwrapped_solana_transaction(params: &Value) -> Value {
    match params.as_array().map(Vec::as_slice) {
        Some([transaction]) => transaction.clone(),
        _ => params.clone(),
    }
}

fn typed_data_with_schema(params: &Value) -> Result<Value, PaymentError> {
    let signer = params.at(0).map_err(invalid)?;
    let typed_data = params.at(1).map_err(invalid)?;
    Ok(Value::Array(vec![signer.clone(), with_domain_schema(typed_data)?]))
}

fn with_domain_schema(typed_data: &Value) -> Result<Value, PaymentError> {
    let Value::String(json) = typed_data else {
        return insert_domain_schema(typed_data);
    };
    let decoded = serde_json::from_str(json).map_err(|error| invalid(format!("Invalid typed data: {error}")))?;
    let encoded = serde_json::to_string(&insert_domain_schema(&decoded)?).map_err(invalid)?;
    Ok(Value::String(encoded))
}

fn insert_domain_schema(typed_data: &Value) -> Result<Value, PaymentError> {
    if typed_data.get_value("types").and_then(|types| types.get_value(TYPE_EIP712_DOMAIN)).is_ok() {
        return Ok(typed_data.clone());
    }

    let domain = typed_data.get_value("domain").map_err(invalid)?;
    let schema = serde_json::to_value(domain_schema(domain)?).map_err(invalid)?;
    let types = object(typed_data.get_value("types").map_err(invalid)?, "types")?
        .clone()
        .into_iter()
        .chain([(TYPE_EIP712_DOMAIN.to_string(), schema)])
        .collect();

    Ok(Value::Object(
        object(typed_data, "typed data")?
            .clone()
            .into_iter()
            .chain([("types".to_string(), Value::Object(types))])
            .collect(),
    ))
}

fn domain_schema(domain: &Value) -> Result<Vec<EIP712Type>, PaymentError> {
    let domain = object(domain, "EIP712 domain")?;
    let fields = eip712_domain_types();

    for (name, value) in domain {
        if !fields.iter().any(|field| field.name == *name) {
            return Err(invalid(format!("Unsupported EIP712 domain field: {name}")));
        }
        if value.is_null() {
            return Err(invalid(format!("Missing EIP712 domain field value: {name}")));
        }
    }

    Ok(fields.into_iter().filter(|field| domain.contains_key(&field.name)).collect())
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, PaymentError> {
    value.as_object().ok_or_else(|| invalid(format!("Expected {name} object")))
}

fn invalid(message: impl Display) -> PaymentError {
    PaymentError::InvalidRequest(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer_params(typed_data: Value) -> Value {
        Value::Array(vec![Value::String("0x1".to_string()), typed_data])
    }

    fn domain_schema_of(params: &Value) -> Value {
        let typed_data = match &params[1] {
            Value::String(json) => serde_json::from_str(json).unwrap(),
            value => value.clone(),
        };
        typed_data["types"][TYPE_EIP712_DOMAIN].clone()
    }

    fn permit2_typed_data() -> Value {
        serde_json::json!({
            "domain": {"name": "Permit2", "chainId": 1, "verifyingContract": "0x00"},
            "types": {"PermitSingle": []},
            "primaryType": "PermitSingle",
            "message": {},
        })
    }

    #[test]
    fn test_map_signer_params() {
        let transaction = Value::String("base64".to_string());
        let solana_params = Value::Array(vec![transaction.clone()]);
        assert_eq!(map_signer_params("solana_signTransaction", &solana_params).unwrap(), transaction);
        assert_eq!(map_signer_params("eth_sendTransaction", &solana_params).unwrap(), solana_params);

        let params = map_signer_params(METHOD_ETH_SIGN_TYPED_DATA_V4, &signer_params(permit2_typed_data())).unwrap();
        assert_eq!(
            domain_schema_of(&params),
            serde_json::json!([
                {"name": "name", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"},
            ])
        );

        let as_string = Value::String(serde_json::to_string(&permit2_typed_data()).unwrap());
        let params = map_signer_params(METHOD_ETH_SIGN_TYPED_DATA_V4, &signer_params(as_string)).unwrap();
        assert!(params[1].is_string());
        assert_eq!(domain_schema_of(&params).as_array().unwrap().len(), 3);

        let with_schema = signer_params(serde_json::json!({
            "domain": {"name": "Permit2"},
            "types": {TYPE_EIP712_DOMAIN: [{"name": "verifyingContract", "type": "address"}]},
        }));
        assert_eq!(map_signer_params(METHOD_ETH_SIGN_TYPED_DATA_V4, &with_schema).unwrap(), with_schema);
    }

    #[test]
    fn test_wallet_connect_params_rejects_unusable_typed_data() {
        let rejected = |typed_data: Value| map_signer_params(METHOD_ETH_SIGN_TYPED_DATA_V4, &signer_params(typed_data)).is_err();

        assert!(rejected(serde_json::json!({"domain": {"chainId": 1, "unexpected": "1"}, "types": {}})));
        assert!(rejected(serde_json::json!({"domain": {"chainId": 1, "salt": "0x00"}, "types": {}})));
        assert!(rejected(serde_json::json!({"domain": {"chainId": null}, "types": {}})));
        assert!(rejected(serde_json::json!({"domain": {"chainId": 1}})));
        assert!(rejected(Value::String("{".to_string())));
    }
}
