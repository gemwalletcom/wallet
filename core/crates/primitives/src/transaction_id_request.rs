use serde::{Deserialize, Deserializer, Serialize};

use crate::{CHAIN_SEPARATOR, Chain, SwapProvider, TransactionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransactionIdRequest {
    pub chain: Chain,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_provider: Option<SwapProvider>,
}

impl TransactionIdRequest {
    pub fn new(chain: Chain, hash: String, block_number: Option<u64>) -> Self {
        Self {
            chain,
            hash,
            block_number,
            swap_provider: None,
        }
    }

    pub fn with_swap_provider(self, swap_provider: Option<SwapProvider>) -> Self {
        Self { swap_provider, ..self }
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
                #[serde(default)]
                swap_provider: Option<SwapProvider>,
            },
            Id(TransactionId),
        }

        Ok(match Input::deserialize(deserializer)? {
            Input::Request { chain, hash, block_number, swap_provider } => Self::new(chain, hash, block_number).with_swap_provider(swap_provider),
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

        let request = TransactionIdRequest::new(Chain::Solana, "abc".to_string(), None).with_swap_provider(Some(SwapProvider::Mayan));
        assert_eq!(serde_json::to_string(&request).unwrap(), r#"{"chain":"solana","hash":"abc","swap_provider":"mayan"}"#);
        assert_eq!(serde_json::from_str::<TransactionIdRequest>(r#"{"chain":"solana","hash":"abc","swap_provider":"mayan"}"#).unwrap(), request);
    }
}
