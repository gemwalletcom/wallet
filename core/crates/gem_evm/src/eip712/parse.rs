use alloy_primitives::{Address, I256, U256, hex};
use primitives::SignerError;
use serde_json::{Map, Value};
use std::str::FromStr;

pub const MAX_WORD_BYTES: usize = 32;

pub enum DecodedValue<'a> {
    Array { element_type: String, items: &'a [Value] },
    Struct { type_name: &'a str, fields: &'a Map<String, Value> },
    String(&'a str),
    Bytes(Vec<u8>),
    FixedBytes(Vec<u8>),
    Bool(bool),
    Address(Address),
    Uint(U256),
    Int(I256),
}

pub fn decode_value<'a>(type_name: &'a str, value: &'a Value, is_struct: impl Fn(&str) -> bool) -> Result<DecodedValue<'a>, SignerError> {
    if value.is_null() {
        return SignerError::invalid_input_err(format!("Missing value for type '{type_name}'"));
    }

    if let Some((element_type, items)) = parse_array(type_name, value)? {
        return Ok(DecodedValue::Array { element_type, items });
    }

    if is_struct(type_name) {
        let fields = value.as_object().ok_or_else(|| SignerError::invalid_input(format!("Expected object for type '{type_name}', got {value}")))?;
        return Ok(DecodedValue::Struct { type_name, fields });
    }

    match type_name {
        "string" => value.as_str().map(DecodedValue::String).ok_or_else(|| SignerError::invalid_input(format!("Expected string value, got {value}"))),
        "bytes" => Ok(DecodedValue::Bytes(decode_hex(type_name, value)?)),
        "bool" => Ok(DecodedValue::Bool(decode_bool(value)?)),
        "address" => {
            let bytes = decode_hex(type_name, value)?;
            if bytes.len() != 20 {
                return SignerError::invalid_input_err(format!("Invalid address length for '{value}'"));
            }
            Ok(DecodedValue::Address(Address::from_slice(&bytes)))
        }
        _ if type_name.starts_with("uint") => {
            let bits = parse_numeric_bits(type_name, "uint")?;
            let number = parse_uint_value(value)?;
            if number.bit_len() > bits {
                return SignerError::invalid_input_err(format!("Value out of range for type '{type_name}' ({bits}-bit unsigned integer)"));
            }
            Ok(DecodedValue::Uint(number))
        }
        _ if type_name.starts_with("int") => {
            let bits = parse_numeric_bits(type_name, "int")?;
            Ok(DecodedValue::Int(check_signed_range(parse_int_value(value)?, bits)?))
        }
        _ if type_name.starts_with("bytes") => {
            let size = parse_fixed_bytes_size(type_name)?;
            let bytes = decode_hex(type_name, value)?;
            if bytes.len() > size {
                return SignerError::invalid_input_err(format!("Value too large for type '{type_name}'"));
            }
            Ok(DecodedValue::FixedBytes(bytes))
        }
        _ => SignerError::invalid_input_err(format!("Unsupported EIP-712 type '{type_name}'")),
    }
}

pub fn validate_eip712_domain_chain_id_binding(value: &Value) -> Result<(), SignerError> {
    let domain_chain_id = value.get("domain").and_then(|domain| domain.get("chainId"));
    let schema_has_chain_id = value
        .get("types")
        .and_then(|types| types.get("EIP712Domain"))
        .and_then(Value::as_array)
        .is_some_and(|fields| fields.iter().any(|field| field.get("name").and_then(Value::as_str) == Some("chainId")));

    match (domain_chain_id, schema_has_chain_id) {
        (Some(_), false) => SignerError::invalid_input_err("EIP712Domain type schema must declare chainId when domain.chainId is present"),
        (Some(Value::Null) | None, true) => SignerError::invalid_input_err("EIP712 domain missing chainId"),
        _ => Ok(()),
    }
}

fn decode_hex(type_name: &str, value: &Value) -> Result<Vec<u8>, SignerError> {
    let string = value.as_str().ok_or_else(|| SignerError::invalid_input(format!("Expected hex string for {type_name}, got {value}")))?;
    hex::decode(string).map_err(SignerError::from_display)
}

fn decode_bool(value: &Value) -> Result<bool, SignerError> {
    match value {
        Value::Bool(b) => Ok(*b),
        Value::Number(num) => match (num.as_u64(), num.as_i64()) {
            (Some(v), _) => Ok(v != 0),
            (_, Some(v)) => Ok(v != 0),
            _ => SignerError::invalid_input_err("Invalid numeric value for bool"),
        },
        other => SignerError::invalid_input_err(format!("Expected boolean value, got {other}")),
    }
}

fn parse_array<'a>(type_name: &str, value: &'a Value) -> Result<Option<(String, &'a [Value])>, SignerError> {
    let Some((element_type, expected_len)) = parse_array_type(type_name)? else {
        return Ok(None);
    };
    let items = value.as_array().ok_or_else(|| SignerError::invalid_input(format!("Expected array for type '{type_name}', got {value}")))?;
    if let Some(len) = expected_len
        && items.len() != len
    {
        return SignerError::invalid_input_err(format!("Expected array of length {len} for type '{type_name}', got {}", items.len()));
    }
    Ok(Some((element_type, items)))
}

fn parse_array_type(type_name: &str) -> Result<Option<(String, Option<usize>)>, SignerError> {
    if !type_name.ends_with(']') {
        if type_name.contains('[') {
            return SignerError::invalid_input_err(format!("Malformed array type '{type_name}'"));
        }
        return Ok(None);
    }

    let start = type_name.rfind('[').ok_or_else(|| SignerError::invalid_input(format!("Malformed array type '{type_name}'")))?;
    let length_str = &type_name[start + 1..type_name.len() - 1];
    let element_type = type_name[..start].to_string();
    if element_type.is_empty() {
        return SignerError::invalid_input_err(format!("Malformed array type '{type_name}'"));
    }

    let length = if length_str.is_empty() {
        None
    } else {
        Some(length_str.parse().map_err(|_| SignerError::invalid_input(format!("Invalid array length for type '{type_name}'")))?)
    };

    Ok(Some((element_type, length)))
}

pub fn base_type_name(type_name: &str) -> &str {
    type_name.split('[').next().unwrap_or(type_name)
}

fn parse_numeric_bits(type_name: &str, prefix: &str) -> Result<usize, SignerError> {
    let bits_part = &type_name[prefix.len()..];
    if bits_part.is_empty() {
        return Ok(256);
    }

    let bits = bits_part.parse::<usize>().map_err(|_| SignerError::invalid_input(format!("Invalid bit size for type '{type_name}'")))?;
    if bits == 0 || bits > MAX_WORD_BYTES * 8 || bits % 8 != 0 {
        return SignerError::invalid_input_err(format!("Unsupported bit size for type '{type_name}'"));
    }
    Ok(bits)
}

fn parse_fixed_bytes_size(type_name: &str) -> Result<usize, SignerError> {
    let size_part = &type_name["bytes".len()..];
    if size_part.is_empty() {
        return SignerError::invalid_input_err(format!("Invalid fixed bytes type '{type_name}'"));
    }

    let size = size_part.parse::<usize>().map_err(|_| SignerError::invalid_input(format!("Invalid length for {type_name}")))?;
    if size == 0 || size > MAX_WORD_BYTES {
        return SignerError::invalid_input_err(format!("Unsupported length for {type_name}"));
    }
    Ok(size)
}

pub fn parse_uint_str(value: &str) -> Result<U256, SignerError> {
    U256::from_str(value).map_err(SignerError::from_display)
}

fn parse_uint_value(value: &Value) -> Result<U256, SignerError> {
    match value {
        Value::String(s) => parse_uint_str(s),
        Value::Number(num) => {
            if let Some(u) = num.as_u64() {
                return Ok(U256::from(u));
            }

            if let Some(i) = num.as_i64() {
                if i >= 0 {
                    return Ok(U256::from(i as u64));
                }
                return SignerError::invalid_input_err("Negative numeric value provided for unsigned integer");
            }

            Err(SignerError::invalid_input("Unsupported numeric value for unsigned integer"))
        }
        other => Err(SignerError::invalid_input(format!("Expected integer value, got {}", other))),
    }
}

fn parse_int_value(value: &Value) -> Result<I256, SignerError> {
    match value {
        Value::String(s) => I256::from_str(s).map_err(SignerError::from_display),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                return I256::try_from(i as i128).map_err(SignerError::from_display);
            }

            if let Some(u) = num.as_u64() {
                return Ok(I256::from_raw(U256::from(u)));
            }

            Err(SignerError::invalid_input("Unsupported numeric value for signed integer"))
        }
        other => Err(SignerError::invalid_input(format!("Expected integer value, got {}", other))),
    }
}

fn check_signed_range(number: I256, bits: usize) -> Result<I256, SignerError> {
    if bits == 0 || bits > MAX_WORD_BYTES * 8 {
        return SignerError::invalid_input_err(format!("Unsupported bit size {bits} for signed integer"));
    }

    if number.bits() > bits as u32 {
        return SignerError::invalid_input_err(format!("Value out of range for signed integer with {bits} bits"));
    }

    Ok(number)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::I256;

    use super::{MAX_WORD_BYTES, check_signed_range};

    #[test]
    fn test_check_signed_range() {
        let negative = check_signed_range(I256::try_from(-42).unwrap(), 32).unwrap().into_raw().to_be_bytes::<MAX_WORD_BYTES>();
        assert_eq!(negative[..31], [0xff; 31]);
        assert_eq!(negative[31], 0xd6);

        let positive = check_signed_range(I256::try_from(42).unwrap(), 32).unwrap().into_raw().to_be_bytes::<MAX_WORD_BYTES>();
        assert_eq!(positive[..31], [0; 31]);
        assert_eq!(positive[31], 42);

        assert!(check_signed_range(I256::try_from(-(1i128 << 31)).unwrap(), 32).is_ok());
        assert!(check_signed_range(I256::try_from(1i128 << 31).unwrap(), 32).is_err());
        assert!(check_signed_range(I256::try_from(-(1i128 << 31) - 1).unwrap(), 32).is_err());
    }
}
