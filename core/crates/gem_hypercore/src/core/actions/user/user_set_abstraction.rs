use crate::core::actions::{MAINNET, SIGNATURE_CHAIN_ID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserSetAbstraction {
    pub abstraction: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub r#type: String,
    pub user: String,
}

impl UserSetAbstraction {
    pub fn unified_account(user: &str, nonce: u64) -> Self {
        Self {
            abstraction: "unifiedAccount".to_string(),
            hyperliquid_chain: MAINNET.to_string(),
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            r#type: "userSetAbstraction".to_string(),
            user: user.to_lowercase(),
        }
    }
}
