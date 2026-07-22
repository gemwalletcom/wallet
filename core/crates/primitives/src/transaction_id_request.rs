use serde::{Deserialize, Deserializer, Serialize};

use crate::{CHAIN_SEPARATOR, Chain, TransactionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransactionIdRequest {
    pub chain: Chain,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
}

impl TransactionIdRequest {
    pub fn new(chain: Chain, hash: String, block_number: Option<u64>) -> Self {
        Self { chain, hash, block_number }
    }
}

impl From<TransactionId> for TransactionIdRequest {
    fn from(id: TransactionId) -> Self {
        Self::new(id.chain, id.hash, None)
    }
}

impl std::fmt::Display for TransactionIdRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}{CHAIN_SEPARATOR}{}", self.chain.as_ref(), self.hash)
    }
}

impl<'de> Deserialize<'de> for TransactionIdRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            Request {
                chain: Chain,
                hash: String,
                #[serde(default)]
                block_number: Option<u64>,
            },
            Id(TransactionId),
        }

        Ok(match Input::deserialize(deserializer)? {
            Input::Request { chain, hash, block_number } => Self::new(chain, hash, block_number),
            Input::Id(id) => id.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_current_and_legacy_queue_payloads() {
        let request = TransactionIdRequest::new(Chain::Ethereum, "0x123".to_string(), Some(42));
        assert_eq!(serde_json::to_string(&request).unwrap(), r#"{"chain":"ethereum","hash":"0x123","block_number":42}"#);
        assert_eq!(
            serde_json::from_str::<TransactionIdRequest>(r#""ethereum_0x123""#).unwrap(),
            TransactionIdRequest::new(Chain::Ethereum, "0x123".to_string(), None)
        );
    }
}
