use alloy_primitives::hex;
use gem_hash::keccak::keccak256;
use serde_json::Value;

pub fn action_hash(action: &Value, vault_address: Option<&str>, nonce: u64, expires_after: Option<u64>) -> Result<String, String> {
    let mut data = Vec::new();
    write_msgpack(&mut data, action)?;
    data.extend_from_slice(&nonce.to_be_bytes());

    if let Some(vault) = vault_address {
        data.push(0x01);
        let vault_bytes = hex::decode(vault).map_err(|e| format!("Invalid vault address: {e}"))?;
        data.extend_from_slice(&vault_bytes);
    } else {
        data.push(0x00);
    }

    if let Some(expires) = expires_after {
        data.push(0x00);
        data.extend_from_slice(&expires.to_be_bytes());
    }

    Ok(hex::encode(keccak256(&data)))
}

fn write_msgpack(out: &mut Vec<u8>, value: &Value) -> Result<(), String> {
    match value {
        Value::Null => out.push(0xc0),
        Value::Bool(false) => out.push(0xc2),
        Value::Bool(true) => out.push(0xc3),
        Value::Number(number) => write_uint(out, number.as_u64().ok_or_else(|| format!("Unsupported action number: {number}"))?),
        Value::String(string) => write_string(out, string),
        Value::Array(values) => {
            write_collection_len(out, values.len(), [0x90, 0xdc, 0xdd]);
            for value in values {
                write_msgpack(out, value)?;
            }
        }
        Value::Object(map) => {
            write_collection_len(out, map.len(), [0x80, 0xde, 0xdf]);
            for (key, value) in map {
                write_string(out, key);
                write_msgpack(out, value)?;
            }
        }
    }
    Ok(())
}

fn write_uint(out: &mut Vec<u8>, value: u64) {
    match value {
        0..128 => out.push(value as u8),
        128..256 => out.extend_from_slice(&[0xcc, value as u8]),
        256..65_536 => {
            out.push(0xcd);
            out.extend_from_slice(&(value as u16).to_be_bytes());
        }
        65_536..4_294_967_296 => {
            out.push(0xce);
            out.extend_from_slice(&(value as u32).to_be_bytes());
        }
        _ => {
            out.push(0xcf);
            out.extend_from_slice(&value.to_be_bytes());
        }
    }
}

fn write_string(out: &mut Vec<u8>, string: &str) {
    let len = string.len();
    match len {
        0..32 => out.push(0xa0 | len as u8),
        32..256 => out.extend_from_slice(&[0xd9, len as u8]),
        256..65_536 => {
            out.push(0xda);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        }
        _ => {
            out.push(0xdb);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
    }
    out.extend_from_slice(string.as_bytes());
}

fn write_collection_len(out: &mut Vec<u8>, len: usize, markers: [u8; 3]) {
    match len {
        0..16 => out.push(markers[0] | len as u8),
        16..65_536 => {
            out.push(markers[1]);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        }
        _ => {
            out.push(markers[2]);
            out.extend_from_slice(&(len as u32).to_be_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_write_msgpack() {
        let address = "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc";
        for (value, expected) in [
            (json!(null), "c0".to_string()),
            (json!(false), "c2".to_string()),
            (json!(true), "c3".to_string()),
            (json!(5), "05".to_string()),
            (json!(127), "7f".to_string()),
            (json!(200), "ccc8".to_string()),
            (json!(10_000), "cd2710".to_string()),
            (json!(110_000), "ce0001adb0".to_string()),
            (json!(1_790_829_872_053u64), "cf000001a0f5c743b5".to_string()),
            (json!("na"), "a26e61".to_string()),
            (json!(address), format!("d92a{}", hex::encode(address))),
            (json!("a".repeat(256)), format!("da0100{}", "61".repeat(256))),
            (json!([1, 2]), "920102".to_string()),
            (json!(vec![1; 16]), format!("dc0010{}", "01".repeat(16))),
            (json!({"a": 4, "b": true}), "82a16104a162c3".to_string()),
        ] {
            let mut out = Vec::new();
            write_msgpack(&mut out, &value).unwrap();
            assert_eq!(hex::encode(out), expected, "{value}");
        }

        for value in [json!(-1), json!(1.5)] {
            assert_eq!(write_msgpack(&mut Vec::new(), &value), Err(format!("Unsupported action number: {value}")));
        }
    }
}
