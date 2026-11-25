mod data;
mod hash_impl;
mod parse;

#[cfg(test)]
mod hash_tests;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_option_u64_from_str_or_int;
use std::collections::HashMap;

use crate::u256::u256_to_biguint;
use parse::{DecodedValue, decode_value, parse_uint_str, validate_eip712_domain_chain_id_binding};

pub use hash_impl::hash_typed_data;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Domain {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "chainId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub chain_id: Option<u64>,
    #[serde(rename = "verifyingContract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifying_contract: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Type {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Field {
    pub name: String,
    pub value: EIP712TypedValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EIP712TypedValue {
    Address { value: String },
    Uint256 { value: String },
    Int256 { value: String },
    String { value: String },
    Bool { value: bool },
    Bytes { value: Vec<u8> },
    Struct { fields: Vec<EIP712Field> },
    Array { items: Vec<EIP712TypedValue> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EIP712Message {
    pub domain: EIP712Domain,
    pub primary_type: String,
    pub message: Vec<EIP712Field>,
}

pub fn find_field_string(fields: &[EIP712Field], name: &str) -> Option<String> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Address { value } | EIP712TypedValue::Uint256 { value } | EIP712TypedValue::String { value } => Some(value.clone()),
        EIP712TypedValue::Struct { .. } | EIP712TypedValue::Int256 { .. } | EIP712TypedValue::Bool { .. } | EIP712TypedValue::Bytes { .. } | EIP712TypedValue::Array { .. } => None,
    })
}

pub fn find_field_biguint(fields: &[EIP712Field], name: &str) -> Option<BigUint> {
    parse_biguint(&find_field_string(fields, name)?)
}

pub fn parse_biguint(value: &str) -> Option<BigUint> {
    parse_uint_str(value).ok().map(|value| u256_to_biguint(&value))
}

pub fn find_field_bool(fields: &[EIP712Field], name: &str) -> Option<bool> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Bool { value } => Some(*value),
        EIP712TypedValue::Address { .. }
        | EIP712TypedValue::Uint256 { .. }
        | EIP712TypedValue::Int256 { .. }
        | EIP712TypedValue::String { .. }
        | EIP712TypedValue::Struct { .. }
        | EIP712TypedValue::Bytes { .. }
        | EIP712TypedValue::Array { .. } => None,
    })
}

pub fn find_field_struct<'a>(fields: &'a [EIP712Field], name: &str) -> Option<&'a [EIP712Field]> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Struct { fields } => Some(fields.as_slice()),
        EIP712TypedValue::Address { .. }
        | EIP712TypedValue::Uint256 { .. }
        | EIP712TypedValue::Int256 { .. }
        | EIP712TypedValue::String { .. }
        | EIP712TypedValue::Bool { .. }
        | EIP712TypedValue::Bytes { .. }
        | EIP712TypedValue::Array { .. } => None,
    })
}

pub fn eip712_domain_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "name".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "version".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "chainId".into(),
            r#type: "uint256".into(),
        },
        EIP712Type {
            name: "verifyingContract".into(),
            r#type: "address".into(),
        },
    ]
}

pub fn validate_eip712_chain_id(data: &str, expected_chain_id: u64) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(data).map_err(|e| format!("Invalid EIP712 JSON: {}", e))?;
    let message = parse_eip712_json(&value)?;

    if let Some(chain_id) = message.domain.chain_id
        && chain_id != expected_chain_id
    {
        return Err(format!("Chain ID mismatch: expected {}, got {}", expected_chain_id, chain_id));
    }

    Ok(())
}

pub fn parse_eip712_json(value: &Value) -> Result<EIP712Message, String> {
    validate_eip712_domain_chain_id_binding(value).map_err(|error| error.to_string())?;
    let domain_value = value.get("domain").ok_or_else(|| "Invalid EIP712 JSON: missing domain".to_string())?;

    let types_value = value.get("types").and_then(Value::as_object).ok_or_else(|| "Invalid EIP712 JSON: missing or invalid types".to_string())?;
    let all_types: HashMap<String, Vec<EIP712Type>> = types_value
        .iter()
        .map(|(name, fields_json)| {
            serde_json::from_value(fields_json.clone())
                .map(|fields| (name.clone(), fields))
                .map_err(|e| format!("Invalid EIP712 JSON: types field '{name}' parse error: {e}"))
        })
        .collect::<Result<_, _>>()?;

    let domain = parse_domain(domain_value, &all_types)?;

    let primary_type_name = value.get("primaryType").and_then(Value::as_str).ok_or_else(|| "Invalid EIP712 JSON: missing or invalid primaryType".to_string())?;

    let message_json_value = value.get("message").ok_or_else(|| "Invalid EIP712 JSON: missing message".to_string())?;

    let message_typed_value = parse_value(primary_type_name, message_json_value, &all_types)?;

    let message_fields = match message_typed_value {
        EIP712TypedValue::Struct { fields } => fields,
        _ => return Err(format!("Primary type '{primary_type_name}' did not resolve to a Struct")),
    };

    Ok(EIP712Message {
        domain,
        primary_type: primary_type_name.to_string(),
        message: message_fields,
    })
}

fn parse_domain(domain_value: &Value, all_types: &HashMap<String, Vec<EIP712Type>>) -> Result<EIP712Domain, String> {
    let mut domain = EIP712Domain {
        name: None,
        version: None,
        chain_id: None,
        verifying_contract: None,
    };
    if !all_types.contains_key("EIP712Domain") {
        return Ok(domain);
    }
    let EIP712TypedValue::Struct { fields } = parse_value("EIP712Domain", domain_value, all_types)? else {
        return Err("EIP712Domain did not resolve to a Struct".to_string());
    };
    for field in fields {
        match (field.name.as_str(), field.value) {
            ("name", EIP712TypedValue::String { value }) => domain.name = Some(value),
            ("version", EIP712TypedValue::String { value }) => domain.version = Some(value),
            ("chainId", EIP712TypedValue::Uint256 { value } | EIP712TypedValue::String { value }) => {
                domain.chain_id = Some(
                    parse_uint_str(&value)
                        .ok()
                        .and_then(|chain_id| u64::try_from(chain_id).ok())
                        .ok_or_else(|| format!("Invalid EIP712 domain chainId '{value}'"))?,
                )
            }
            ("verifyingContract", EIP712TypedValue::Address { value }) => domain.verifying_contract = Some(value),
            ("name" | "version" | "chainId" | "verifyingContract", _) => return Err(format!("Unsupported type for EIP712 domain field '{}'", field.name)),
            _ => {}
        }
    }
    Ok(domain)
}

pub fn parse_value(type_name: &str, json_value: &Value, all_types: &HashMap<String, Vec<EIP712Type>>) -> Result<EIP712TypedValue, String> {
    let decoded = decode_value(type_name, json_value, |name| all_types.contains_key(name)).map_err(|error| error.to_string())?;
    Ok(match decoded {
        DecodedValue::Array { element_type, items } => EIP712TypedValue::Array {
            items: items.iter().map(|item| parse_value(&element_type, item, all_types)).collect::<Result<_, _>>()?,
        },
        DecodedValue::Struct { type_name, fields } => {
            let defined_fields = all_types.get(type_name).ok_or_else(|| format!("Unknown EIP-712 type '{type_name}'"))?;
            let fields = defined_fields
                .iter()
                .map(|field| {
                    let value = fields.get(&field.name).ok_or_else(|| format!("Missing field '{}' for struct type '{type_name}'", field.name))?;
                    Ok(EIP712Field {
                        name: field.name.clone(),
                        value: parse_value(&field.r#type, value, all_types)?,
                    })
                })
                .collect::<Result<_, String>>()?;
            EIP712TypedValue::Struct { fields }
        }
        DecodedValue::String(value) => EIP712TypedValue::String { value: value.to_string() },
        DecodedValue::Bytes(value) | DecodedValue::FixedBytes(value) => EIP712TypedValue::Bytes { value },
        DecodedValue::Bool(value) => EIP712TypedValue::Bool { value },
        DecodedValue::Address(address) => EIP712TypedValue::Address { value: address.to_checksum(None) },
        DecodedValue::Uint(number) => EIP712TypedValue::Uint256 { value: number.to_string() },
        DecodedValue::Int(number) => EIP712TypedValue::Int256 { value: number.to_string() },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::eip712_mock::mock_eip712_json;
    use primitives::{asset_constants::ETHEREUM_USDT_TOKEN_ID, contract_constants::ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT};

    #[test]
    fn test_permit2_json_parsing() {
        let value: serde_json::Value = serde_json::from_str(include_str!("../testdata/uniswap_permit2.json")).unwrap();
        let result = parse_eip712_json(&value);

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let message = result.unwrap();

        assert_eq!(message.domain.chain_id, Some(1));
        assert_eq!(message.message.len(), 3);

        match &message.message[0].value {
            EIP712TypedValue::Struct { fields } => {
                assert_eq!(fields.len(), 4); // token, amount, expiration, nonce

                // 1.1 token (address)
                assert_eq!(fields[0].name, "token");
                match &fields[0].value {
                    EIP712TypedValue::Address { value } => assert_eq!(value, ETHEREUM_USDT_TOKEN_ID),
                    _ => panic!("Incorrect type for details.token"),
                }
                // 1.2 amount (uint160 - parsed as Uint256 for now)
                // We parse uint160 as Uint256 { value: String } because the JSON value is a string.
                assert_eq!(fields[1].name, "amount");
                match &fields[1].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1461501637330902918203684832716283019655932542975"),
                    _ => panic!("Incorrect type for details.amount"),
                }
                // 1.3 expiration (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[2].name, "expiration");
                match &fields[2].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1732780554"),
                    _ => panic!("Incorrect type for details.expiration"),
                }
                // 1.4 nonce (uint48 - parsed as Uint256 for now)
                assert_eq!(fields[3].name, "nonce");
                match &fields[3].value {
                    EIP712TypedValue::Uint256 { value } => assert_eq!(value, "0"),
                    _ => panic!("Incorrect type for details.nonce"),
                }
            }
            _ => panic!("Incorrect type for details field"),
        }

        assert_eq!(message.message[1].name, "spender");
        match &message.message[1].value {
            EIP712TypedValue::Address { value } => {
                assert_eq!(value.to_lowercase(), ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.to_lowercase());
            }
            _ => panic!("Expected spender field to be an Address"),
        }

        assert_eq!(message.message[2].name, "sigDeadline");
        match &message.message[2].value {
            EIP712TypedValue::Uint256 { value } => {
                assert_eq!(value, "1730190354");
            }
            _ => panic!("Expected sigDeadline field to be a Uint256"),
        }
    }

    #[test]
    fn test_find_field() {
        let value: serde_json::Value = serde_json::from_str(include_str!("../testdata/uniswap_permit2.json")).unwrap();
        let message = parse_eip712_json(&value).unwrap();
        let details = find_field_struct(&message.message, "details").unwrap();

        assert_eq!(find_field_string(details, "token"), Some(ETHEREUM_USDT_TOKEN_ID.to_string()));
        assert_eq!(find_field_string(&message.message, "details"), None);
        assert_eq!(find_field_struct(&message.message, "spender"), None);
        assert_eq!(find_field_biguint(details, "amount"), Some(BigUint::parse_bytes(b"1461501637330902918203684832716283019655932542975", 10).unwrap()));
        assert_eq!(
            find_field_biguint(
                &[EIP712Field {
                    name: "amount".to_string(),
                    value: EIP712TypedValue::Uint256 { value: "0x186a0".to_string() }
                }],
                "amount"
            ),
            Some(BigUint::from(100_000u32))
        );
        assert_eq!(find_field_biguint(details, "nonce"), Some(BigUint::ZERO));
        assert_eq!(find_field_biguint(details, "deadline"), None);
    }

    #[test]
    fn test_1inch_permit_json_parsing() {
        let value: serde_json::Value = serde_json::from_str(include_str!("../testdata/1inch_permit.json")).unwrap();
        let result = parse_eip712_json(&value);

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let message = result.unwrap();

        assert_eq!(message.domain.chain_id, Some(1));
        assert!(message.message.len() == 5);
    }

    #[test]
    fn test_validate_eip712_chain_id_match() {
        let result = validate_eip712_chain_id(&mock_eip712_json(1), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_eip712_chain_id_mismatch() {
        let result = validate_eip712_chain_id(&mock_eip712_json(137), 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_validate_eip712_chain_id_rejects_unbound_chain_id() {
        let missing_schema_field = include_str!("../testdata/eip712_domain_chain_id_without_schema_field.json");
        assert!(validate_eip712_chain_id(missing_schema_field, 1).unwrap_err().contains("chainId"));

        let schema_without_domain_value = include_str!("../testdata/eip712_schema_chain_id_without_domain_value.json");
        assert!(validate_eip712_chain_id(schema_without_domain_value, 1).unwrap_err().contains("missing chainId"));

        let null_domain_chain_id = include_str!("../testdata/eip712_domain_chain_id_null_value.json");
        assert!(validate_eip712_chain_id(null_domain_chain_id, 1).unwrap_err().contains("chainId"));
    }

    #[test]
    fn test_validate_eip712_chain_id_accepts_declared_non_uint_chain_id_type() {
        let json = include_str!("../testdata/eip712_domain_chain_id_schema_string.json");
        assert!(validate_eip712_chain_id(json, 1).is_ok());

        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["domain"]["chainId"] = serde_json::json!("0x1");
        assert!(validate_eip712_chain_id(&value.to_string(), 1).is_ok());
        assert!(validate_eip712_chain_id(&value.to_string(), 137).unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_eip712_json_without_domain_chain_id() {
        let json = include_str!("../testdata/ens_upload_avatar.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let result = parse_eip712_json(&value).unwrap();

        assert_eq!(result.domain.name.as_deref(), Some("Ethereum Name Service"));
        assert_eq!(result.domain.version.as_deref(), Some("1"));
        assert_eq!(result.domain.chain_id, None);
        assert_eq!(result.primary_type, "Upload");
        assert_eq!(result.message.len(), 4);
        assert!(validate_eip712_chain_id(json, 1).is_ok());
    }

    #[test]
    fn test_int8_type_parsing() {
        let value: serde_json::Value = serde_json::from_str(include_str!("../testdata/eip712_int8_account_registration.json")).unwrap();
        let result = parse_eip712_json(&value);

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let message = result.unwrap();
        assert_eq!(message.primary_type, "AccountRegistration");
        assert_eq!(message.message.len(), 1);

        assert_eq!(message.message[0].name, "accountIndex");
        match &message.message[0].value {
            EIP712TypedValue::Int256 { value } => assert_eq!(value, "1"),
            _ => panic!("Expected Int256 type for accountIndex field"),
        }
    }

    #[test]
    fn test_fixed_size_array_parsing() {
        let json = include_str!("../testdata/eip712_arrays_nested.json");
        assert!(validate_eip712_chain_id(json, 31337).is_ok());

        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        let message = parse_eip712_json(&value).unwrap();
        let weights = message.message.iter().find(|field| field.name == "weights").unwrap();
        assert_eq!(
            weights.value,
            EIP712TypedValue::Array {
                items: ["1", "2", "3"].map(|value| EIP712TypedValue::Uint256 { value: value.to_string() }).to_vec()
            }
        );

        value["message"]["weights"] = serde_json::json!(["1", "2"]);
        assert!(parse_eip712_json(&value).unwrap_err().contains("Expected array of length 3"));

        value["types"]["Group"][3]["type"] = serde_json::json!("uint64[3][2]");
        value["message"]["weights"] = serde_json::json!([["1", "2", "3"], ["4", "5", "6"]]);
        assert!(parse_eip712_json(&value).is_ok());
    }

    #[test]
    fn test_review_parser_accepts_what_signer_accepts() {
        let base: serde_json::Value = serde_json::from_str(include_str!("../testdata/eip712_arrays_nested.json")).unwrap();
        let cases = [
            ("/message/weights", serde_json::json!(["1", "2"]), false),
            ("/message/weights", serde_json::json!(["18446744073709551616", "2", "3"]), false),
            ("/message/weights", serde_json::json!(["0x10", 2, "3"]), true),
            ("/message/nested/flag", serde_json::json!(1), true),
            ("/message/nested/flag", serde_json::json!("true"), false),
            ("/message/nested/payload", serde_json::json!(format!("0x{}", "01".repeat(33))), false),
            ("/message/members/0", serde_json::json!("0x90f8bf6a479f320ead074411a4b0e7944ea8c9"), false),
            ("/message/name", serde_json::json!(5), false),
            ("/types/Group/3/type", serde_json::json!("uint7[3]"), false),
            ("/domain/verifyingContract", serde_json::json!("0x90f8bf6a479f320ead074411a4b0e7944ea8c9"), false),
            ("/domain/chainId", serde_json::json!("0x7a69"), true),
            ("/domain/chainId", serde_json::json!("abc"), false),
            ("/domain/name", serde_json::json!(5), false),
            ("/types/EIP712Domain", serde_json::json!([{ "name": "name", "type": "string" }]), false),
        ];

        for (pointer, replacement, accepted) in cases {
            let mut value = base.clone();
            *value.pointer_mut(pointer).unwrap() = replacement;
            assert_eq!(parse_eip712_json(&value).is_ok(), accepted, "review {pointer}");
            assert_eq!(hash_typed_data(&value.to_string()).is_ok(), accepted, "signer {pointer}");
        }

        let mut value = base.clone();
        value["domain"]["chainId"] = serde_json::json!("0x7a69");
        value["domain"]["verifyingContract"] = serde_json::json!("0xcccccccccccccccccccccccccccccccccccccccc");
        value["types"]["EIP712Domain"] = serde_json::json!([{ "name": "name", "type": "string" }, { "name": "chainId", "type": "uint256" }]);
        let domain = parse_eip712_json(&value).unwrap().domain;
        assert_eq!(domain.chain_id, Some(31337));
        assert_eq!(domain.verifying_contract, None);

        let mut value = base;
        value["message"]["weights"] = serde_json::json!(["0x10", 2, "3"]);
        let message = parse_eip712_json(&value).unwrap();
        let weights = message.message.iter().find(|field| field.name == "weights").unwrap();
        assert_eq!(
            weights.value,
            EIP712TypedValue::Array {
                items: ["16", "2", "3"].map(|value| EIP712TypedValue::Uint256 { value: value.to_string() }).to_vec()
            }
        );
    }
}
