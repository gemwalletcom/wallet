use std::collections::HashMap;

use num_bigint::BigUint;
use primitives::{SwapProvider, TransactionState};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str};

use crate::address::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedBody {
    pub comment: Option<String>,
    pub text: Option<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTransactions {
    pub transactions: Vec<TransactionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResponse {
    pub traces: Vec<Trace>,
}

impl TraceResponse {
    pub fn root_transaction(&self) -> Option<&TransactionMessage> {
        self.traces.first()?.root_transaction()
    }

    pub fn action_state(&self) -> Option<TransactionState> {
        self.traces.first().map(Trace::action_state)
    }

    pub fn has_actions(&self) -> bool {
        self.traces.first().is_some_and(Trace::has_actions)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceByMessageQuery {
    pub msg_hash: String,
    pub include_actions: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceByTransactionQuery {
    pub tx_hash: String,
    pub include_actions: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceByBlockQuery {
    pub mc_seqno: u64,
    pub include_actions: bool,
    pub limit: usize,
    pub offset: usize,
    pub sort: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceByAddressQuery {
    pub account: String,
    pub include_actions: bool,
    pub limit: usize,
    pub offset: usize,
    pub sort: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub is_incomplete: bool,
    pub actions: Vec<TraceAction>,
    #[serde(default)]
    pub transactions_order: Vec<String>,
    #[serde(default)]
    pub transactions: HashMap<String, TransactionMessage>,
}

impl Trace {
    pub fn root_transaction(&self) -> Option<&TransactionMessage> {
        if let Some(transaction_id) = self.transactions_order.first() {
            return self.transactions.get(transaction_id);
        }

        if self.transactions.len() == 1 {
            return self.transactions.values().next();
        }

        None
    }

    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn action_state(&self) -> TransactionState {
        if self.is_incomplete {
            return TransactionState::Pending;
        }
        for action in &self.actions {
            if action.success == Some(false) {
                return TransactionState::Reverted;
            }
        }
        TransactionState::Confirmed
    }
}

pub const TRACE_ACTION_JETTON_SWAP: &str = "jetton_swap";
pub const TRACE_ACTION_JETTON_TRANSFER: &str = "jetton_transfer";
pub const TRACE_ACTION_NFT_TRANSFER: &str = "nft_transfer";
pub const TRACE_ACTION_TON_TRANSFER: &str = "ton_transfer";
pub const STONFI_PAY_VAULT_V2: &str = "stonfi_pay_vault_v2";
const TRACE_DEX_STONFI: &str = "stonfi";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAction {
    pub success: Option<bool>,
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonSwapDetails {
    pub dex: Option<String>,
    pub sender: String,
    pub asset_in: Option<String>,
    pub asset_out: Option<String>,
    pub dex_incoming_transfer: SwapTransfer,
    pub dex_outgoing_transfer: SwapTransfer,
}

impl JettonSwapDetails {
    pub fn provider(&self) -> Option<String> {
        match self.dex.as_deref() {
            Some(TRACE_DEX_STONFI) => Some(SwapProvider::StonfiV2.id().to_string()),
            _ => self.dex.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwapTransfer {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonTransferDetails {
    pub asset: String,
    pub sender: String,
    pub receiver: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TonTransferDetails {
    pub destination: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NftTransferDetails {
    pub nft_collection: Address,
    pub nft_item: Address,
    pub old_owner: Address,
    pub new_owner: Address,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub hash: String,
    pub now: i64,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub total_fees: BigUint,
    pub description: Option<TransactionDescription>,
    pub out_msgs: Vec<OutMessage>,
    pub in_msg: Option<TransactionInMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub source: String,
    pub destination: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub value: Option<BigUint>,
    #[serde(alias = "opcode")]
    pub op_code: Option<String>,
    pub comment: Option<String>,
    pub decoded_body: Option<DecodedBody>,
    pub message_content: Option<MessageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub hash: Option<String>,
    pub decoded: Option<Box<RawValue>>,
}

#[derive(Deserialize)]
struct TextComment {
    #[serde(rename = "type")]
    body_type: String,
    comment: String,
}

impl MessageContent {
    pub fn text_comment(&self) -> Option<String> {
        let decoded: TextComment = serde_json::from_str(self.decoded.as_ref()?.get()).ok()?;
        (decoded.body_type == "text_comment").then_some(decoded.comment)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInMessage {
    pub hash: Option<String>,
    pub source: Option<String>,
    pub destination: String,
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub value: Option<BigUint>,
    pub opcode: Option<String>,
    pub message_content: Option<MessageContent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StonfiPayVault {
    #[serde(rename = "@type")]
    pub body_type: String,
    pub owner: TlbAddress,
    #[serde(rename = "")]
    pub amounts: StonfiPayVaultAmounts,
}

impl StonfiPayVault {
    pub fn amount(&self) -> BigUint {
        &self.amounts.amount0_out.value + &self.amounts.amount1_out.value
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StonfiPayVaultAmounts {
    pub amount0_out: TlbVarUint,
    pub amount1_out: TlbVarUint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlbAddress {
    pub workchain_id: String,
    pub address: String,
}

impl TlbAddress {
    pub fn raw(&self) -> String {
        format!("{}:{}", self.workchain_id, self.address)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlbVarUint {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDescription {
    pub aborted: bool,
    pub compute_ph: Option<ComputePhase>,
    pub action: Option<ActionPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePhase {
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPhase {
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastTransaction {
    pub message_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_comment() {
        for (response, expected) in [
            (r#"{"decoded":{"type":"text_comment","comment":"memo\n\u2764"}}"#, Some("memo\n❤")),
            (r#"{"decoded":{"type":"binary_comment","comment":"not a memo"}}"#, None),
            (r#"{"decoded":{"@type":"scaled_ui_display_multiplier_changed","comment":{"@type":"nothing_from_maybe"}}}"#, None),
            (r#"{"decoded":{"type":"contract_call","comment":{"value":"not a memo"}}}"#, None),
            (r#"{"decoded":{"type":"text_comment","comment":null}}"#, None),
            (r#"{"decoded":null}"#, None),
            (r#"{}"#, None),
        ] {
            let content: MessageContent = serde_json::from_str(response).unwrap();
            assert_eq!(content.text_comment().as_deref(), expected);
        }
    }

    #[test]
    fn test_text_comment_skips_contract_payload() {
        let nested = format!("{}null{}", "[".repeat(148), "]".repeat(148));
        for (response, expected) in [
            (format!(r#"{{"decoded":{{"@type":"wallet_v5_signed_request","text":{nested},"comment":{nested}}}}}"#), None),
            (format!(r#"{{"decoded":{{"type":"text_comment","comment":"memo","payload":{nested}}}}}"#), Some("memo")),
        ] {
            let content: MessageContent = serde_json::from_str(&response).unwrap();
            assert_eq!(content.text_comment().as_deref(), expected);
        }
    }
}
