use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]> + ?Sized,
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(value.as_ref()))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    decode(&value).map_err(D::Error::custom)
}

fn decode(value: &str) -> Result<Vec<u8>, hex::FromHexError> {
    let value = value.trim();
    let value = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")).unwrap_or(value);
    if value.is_empty() {
        return Ok(vec![]);
    }

    let normalized = if value.len() % 2 == 1 { Cow::Owned(format!("0{value}")) } else { Cow::Borrowed(value) };
    hex::decode(normalized.as_ref())
}

pub mod option {
    use super::*;

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_some(&hex::encode(value)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer)?
            .map(|value| decode(&value).map_err(D::Error::custom))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct BytesValue {
        #[serde(with = "crate::hex_bytes")]
        value: Vec<u8>,
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct OptionBytesValue {
        #[serde(default, skip_serializing_if = "Option::is_none", with = "crate::hex_bytes::option")]
        value: Option<Vec<u8>>,
    }

    #[test]
    fn test_hex_bytes() {
        let value: BytesValue = serde_json::from_str(r#"{"value":" 0xabc "}"#).unwrap();
        assert_eq!(value.value, vec![0x0a, 0xbc]);
        assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"value":"0abc"}"#);
    }

    #[test]
    fn test_option_hex_bytes() {
        let value: OptionBytesValue = serde_json::from_str(r#"{"value":"0x"}"#).unwrap();
        assert_eq!(value.value, Some(vec![]));
        assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"value":""}"#);

        let value: OptionBytesValue = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(value.value, None);
        assert_eq!(serde_json::to_string(&value).unwrap(), r#"{}"#);
    }
}
