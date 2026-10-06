use crate::core::actions::{MAINNET, SIGNATURE_CHAIN_ID};
use serde::{Deserialize, Serialize};

pub const SPOT_DEX: &str = "spot";

#[derive(Clone, Serialize, Deserialize)]
pub struct SendAsset {
    pub r#type: String,
    pub destination: String,
    #[serde(rename = "sourceDex")]
    pub source_dex: String,
    #[serde(rename = "destinationDex")]
    pub destination_dex: String,
    pub token: String,
    pub amount: String,
    #[serde(rename = "fromSubAccount")]
    pub from_sub_account: String,
    pub nonce: u64,
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    #[serde(rename = "hyperliquidChain")]
    pub hyperliquid_chain: String,
}

impl SendAsset {
    pub fn spot(amount: String, destination: String, token: String, nonce: u64) -> Self {
        Self {
            r#type: "sendAsset".to_string(),
            destination: destination.to_lowercase(),
            source_dex: SPOT_DEX.to_string(),
            destination_dex: SPOT_DEX.to_string(),
            token,
            amount,
            from_sub_account: String::new(),
            nonce,
            signature_chain_id: SIGNATURE_CHAIN_ID.to_string(),
            hyperliquid_chain: MAINNET.to_string(),
        }
    }
}
