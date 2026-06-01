use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, SeqAccess, Visitor},
    ser::SerializeSeq,
};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunGetMethodRequest {
    pub address: String,
    pub method: String,
    pub stack: Vec<StackArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackArg {
    Num(String),
    Slice(String),
}

impl StackArg {
    pub fn num(value: impl Into<String>) -> Self {
        Self::Num(value.into())
    }

    pub fn slice(value: impl Into<String>) -> Self {
        Self::Slice(value.into())
    }
}

impl Serialize for StackArg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        match self {
            Self::Num(value) => {
                seq.serialize_element("num")?;
                seq.serialize_element(value)?;
            }
            Self::Slice(value) => {
                seq.serialize_element("tvm.Slice")?;
                seq.serialize_element(value)?;
            }
        }
        seq.end()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RunGetMethodResult {
    pub stack: Vec<StackEntry>,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackEntry {
    Num(String),
    Cell { bytes: String },
    Slice { bytes: String },
    Unsupported { kind: String, value: Value },
}

impl StackEntry {
    pub fn as_num(&self) -> Option<&str> {
        match self {
            Self::Num(value) => Some(value),
            Self::Cell { .. } | Self::Slice { .. } | Self::Unsupported { .. } => None,
        }
    }

    pub fn as_cell_bytes(&self) -> Option<&str> {
        match self {
            Self::Cell { bytes } | Self::Slice { bytes } => Some(bytes),
            Self::Num(_) | Self::Unsupported { .. } => None,
        }
    }
}

impl<'de> Deserialize<'de> for StackEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StackEntryVisitor;

        impl<'de> Visitor<'de> for StackEntryVisitor {
            type Value = StackEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a Toncenter stack entry")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let kind = seq.next_element::<String>()?.ok_or_else(|| DeError::custom("missing stack entry kind"))?;
                let value = seq.next_element::<Value>()?.ok_or_else(|| DeError::custom("missing stack entry value"))?;

                match kind.as_str() {
                    "num" => value
                        .as_str()
                        .map(|value| StackEntry::Num(value.to_string()))
                        .ok_or_else(|| DeError::custom("invalid num stack entry")),
                    "cell" => cell_bytes(value)
                        .map(|bytes| StackEntry::Cell { bytes })
                        .ok_or_else(|| DeError::custom("invalid cell stack entry")),
                    "tvm.Slice" => cell_bytes(value)
                        .map(|bytes| StackEntry::Slice { bytes })
                        .ok_or_else(|| DeError::custom("invalid slice stack entry")),
                    _ => Ok(StackEntry::Unsupported { kind, value }),
                }
            }
        }

        deserializer.deserialize_seq(StackEntryVisitor)
    }
}

fn cell_bytes(value: Value) -> Option<String> {
    match value {
        Value::String(bytes) => Some(bytes),
        Value::Object(mut object) => object.remove("bytes").and_then(|value| value.as_str().map(str::to_string)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_arg_serialization() {
        let request = RunGetMethodRequest {
            address: "EQCrouter".to_string(),
            method: "get_pool_address".to_string(),
            stack: vec![StackArg::num("1000"), StackArg::slice("te6cc")],
        };

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "address": "EQCrouter",
                "method": "get_pool_address",
                "stack": [["num", "1000"], ["tvm.Slice", "te6cc"]],
            })
        );
    }

    #[test]
    fn test_stack_entry_deserialization() {
        let stack: Vec<StackEntry> = serde_json::from_value(serde_json::json!([
            ["num", "0x7"],
            ["cell", {"bytes": "te6cc"}],
            ["tvm.Slice", "te6slice"]
        ]))
        .unwrap();

        assert_eq!(stack[0].as_num(), Some("0x7"));
        assert_eq!(stack[1].as_cell_bytes(), Some("te6cc"));
        assert_eq!(stack[2].as_cell_bytes(), Some("te6slice"));
    }
}
