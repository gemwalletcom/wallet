use gem_hash::keccak::keccak256;
use primitives::SignerError;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

use super::data::{TypeField, TypedData};
use super::parse::{DecodedValue, MAX_WORD_BYTES, base_type_name, decode_value, validate_eip712_domain_chain_id_binding};

const PREFIX_PERSONAL_MESSAGE: &[u8] = b"\x19\x01";

pub fn hash_typed_data(json: &str) -> Result<[u8; 32], SignerError> {
    let value: Value = serde_json::from_str(json).map_err(|err| SignerError::invalid_input(format!("Invalid EIP-712 JSON: {err}")))?;
    validate_eip712_domain_chain_id_binding(&value)?;
    let parsed = TypedData::from_value(value)?;

    if parsed.message.is_null() {
        return SignerError::invalid_input_err("Invalid EIP-712 JSON: missing message");
    }

    let domain_hash = if parsed.types.contains_key("EIP712Domain") {
        hash_struct("EIP712Domain", Some(&parsed.domain), &parsed.types)?
    } else {
        [0u8; 32]
    };

    let message_hash = hash_struct(&parsed.primary_type, Some(&parsed.message), &parsed.types)?;

    let mut preimage = Vec::with_capacity(2 + 32 + 32);
    preimage.extend_from_slice(PREFIX_PERSONAL_MESSAGE);
    preimage.extend_from_slice(&domain_hash);
    preimage.extend_from_slice(&message_hash);

    Ok(keccak256(&preimage))
}

fn hash_struct(primary_type: &str, data: Option<&Value>, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    let fields = types.get(primary_type).ok_or_else(|| SignerError::invalid_input(format!("Unknown EIP-712 type '{primary_type}'")))?;

    let data_map: &Map<String, Value> = match data {
        Some(Value::Object(map)) => map,
        Some(Value::Null) | None => return Err(SignerError::invalid_input(format!("Missing object for type '{primary_type}'"))),
        Some(other) => return Err(SignerError::invalid_input(format!("Expected object for type '{primary_type}', got {}", other))),
    };

    let mut encoded = Vec::with_capacity(32 * (fields.len() + 1));
    let type_hash = hash_type(primary_type, types)?;
    encoded.extend_from_slice(&type_hash);

    for field in fields {
        let encoded_value = encode_value(&field.r#type, data_map.get(&field.name), types)?;
        encoded.extend_from_slice(&encoded_value);
    }

    Ok(keccak256(&encoded))
}

fn encode_value(type_name: &str, value: Option<&Value>, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    let value = value.ok_or_else(|| SignerError::invalid_input(format!("Missing value for type '{type_name}'")))?;

    match decode_value(type_name, value, |name| types.contains_key(name))? {
        DecodedValue::Array { element_type, items } => {
            let mut concatenated = Vec::new();
            for item in items {
                concatenated.extend_from_slice(&encode_value(&element_type, Some(item), types)?);
            }
            Ok(keccak256(&concatenated))
        }
        DecodedValue::Struct { type_name, .. } => hash_struct(type_name, Some(value), types),
        DecodedValue::String(string) => Ok(keccak256(string.as_bytes())),
        DecodedValue::Bytes(bytes) => Ok(keccak256(&bytes)),
        DecodedValue::FixedBytes(bytes) => Ok(right_pad(&bytes)),
        DecodedValue::Bool(value) => Ok(if value { left_pad(&[1]) } else { [0u8; 32] }),
        DecodedValue::Address(address) => Ok(left_pad(address.as_slice())),
        DecodedValue::Uint(number) => Ok(number.to_be_bytes::<MAX_WORD_BYTES>()),
        DecodedValue::Int(number) => Ok(number.into_raw().to_be_bytes::<MAX_WORD_BYTES>()),
    }
}

fn hash_type(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<[u8; 32], SignerError> {
    let encoded = encode_type(primary_type, types)?;
    Ok(keccak256(encoded.as_bytes()))
}

fn encode_type(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>) -> Result<String, SignerError> {
    if !types.contains_key(primary_type) {
        return Err(SignerError::invalid_input(format!("Unknown EIP-712 type '{primary_type}'")));
    }

    let mut deps = BTreeSet::new();
    collect_type_dependencies(primary_type, types, &mut deps);
    deps.remove(primary_type);

    let mut parts = Vec::with_capacity(deps.len() + 1);
    parts.push(primary_type.to_string());
    parts.extend(deps);

    let mut encoded = String::new();
    for type_name in parts {
        let fields = types.get(&type_name).ok_or_else(|| SignerError::invalid_input(format!("Unknown EIP-712 type '{type_name}'")))?;

        encoded.push_str(&type_name);
        encoded.push('(');
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                encoded.push(',');
            }
            encoded.push_str(&field.r#type);
            encoded.push(' ');
            encoded.push_str(&field.name);
        }
        encoded.push(')');
    }

    Ok(encoded)
}

fn collect_type_dependencies(primary_type: &str, types: &std::collections::HashMap<String, Vec<TypeField>>, results: &mut BTreeSet<String>) {
    let base = base_type_name(primary_type);
    if results.contains(base) || !types.contains_key(base) {
        return;
    }

    results.insert(base.to_string());

    if let Some(fields) = types.get(base) {
        for field in fields {
            collect_type_dependencies(&field.r#type, types, results);
        }
    }
}

fn left_pad(bytes: &[u8]) -> [u8; MAX_WORD_BYTES] {
    let mut out = [0u8; MAX_WORD_BYTES];
    let len = bytes.len().min(MAX_WORD_BYTES);
    out[MAX_WORD_BYTES - len..].copy_from_slice(&bytes[bytes.len() - len..]);
    out
}

fn right_pad(bytes: &[u8]) -> [u8; MAX_WORD_BYTES] {
    let mut out = [0u8; MAX_WORD_BYTES];
    let len = bytes.len().min(MAX_WORD_BYTES);
    out[..len].copy_from_slice(&bytes[..len]);
    out
}
