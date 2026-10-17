#[cfg(feature = "tvm")]
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[cfg(feature = "tvm")]
use crate::{address::Address, tvm::TvmError};

#[derive(Debug, Clone, Serialize)]
pub struct SendBocRequest {
    pub boc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunGetMethodRequest {
    pub address: String,
    pub method: String,
    pub stack: Vec<StackArg>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunGetMethodResult {
    pub stack: Vec<StackEntry>,
    pub exit_code: i32,
}

#[cfg(feature = "tvm")]
impl RunGetMethodResult {
    pub fn stack_num(&self, index: usize) -> Result<BigUint, TvmError> {
        let value = self.stack.get(index).and_then(StackEntry::as_num).ok_or_else(|| TvmError::new("missing TON number stack entry"))?;
        match value.strip_prefix("0x") {
            Some(hex) => BigUint::parse_bytes(hex.as_bytes(), 16),
            None => BigUint::parse_bytes(value.as_bytes(), 10),
        }
        .ok_or_else(|| TvmError::new(format!("invalid TON stack number {value}")))
    }

    pub fn stack_address(&self, index: usize) -> Result<Address, TvmError> {
        let bytes = self.stack.get(index).and_then(StackEntry::as_cell_bytes).ok_or_else(|| TvmError::new("missing TON address stack cell"))?;
        Address::from_boc_base64(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum StackEntry {
    Num(String),
    Cell(String),
    Slice(String),
    #[serde(other)]
    Unsupported,
}

impl StackEntry {
    pub fn as_num(&self) -> Option<&str> {
        match self {
            Self::Num(value) => Some(value),
            Self::Cell(_) | Self::Slice(_) | Self::Unsupported => None,
        }
    }

    pub fn as_cell_bytes(&self) -> Option<&str> {
        match self {
            Self::Cell(bytes) | Self::Slice(bytes) => Some(bytes),
            Self::Num(_) | Self::Unsupported => None,
        }
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
        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"address":"EQCrouter","method":"get_pool_address","stack":[{"type":"num","value":"1000"},{"type":"slice","value":"te6cc"}]}"#
        );
    }

    #[test]
    fn test_stack_entry_deserialization() {
        let stack: Vec<StackEntry> = serde_json::from_str(r#"[{"type":"num","value":"0x7"},{"type":"cell","value":"te6cc"},{"type":"slice","value":"te6slice"},{"type":"null"}]"#).unwrap();
        assert_eq!(stack, vec![StackEntry::Num("0x7".into()), StackEntry::Cell("te6cc".into()), StackEntry::Slice("te6slice".into()), StackEntry::Unsupported]);
        assert_eq!(stack[0].as_num(), Some("0x7"));
        assert_eq!(stack[1].as_cell_bytes(), Some("te6cc"));
        assert_eq!(stack[2].as_cell_bytes(), Some("te6slice"));
        assert_eq!(stack[3].as_cell_bytes(), None);
        for invalid in [r#"{"type":"num","value":7}"#, r#"{"type":"cell"}"#] {
            assert!(serde_json::from_str::<StackEntry>(invalid).is_err());
        }
    }
}
