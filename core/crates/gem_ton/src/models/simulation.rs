use std::collections::HashMap;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use crate::Address;

use super::{TransactionDescription, wallet_connect::TonConnectMessage};

#[derive(Serialize)]
pub(crate) struct TonEmulationRequest<'a> {
    pub(crate) from: &'a str,
    pub(crate) messages: &'a [TonConnectMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) valid_until: Option<u64>,
    pub(crate) with_actions: bool,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationResponse {
    pub(crate) actions: Vec<TonEmulationAction>,
    pub(crate) transactions: HashMap<String, TonEmulationTransaction>,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationAction {
    pub(crate) success: Option<bool>,
    #[serde(flatten)]
    pub(crate) action: TonEmulationActionType,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "details", rename_all = "snake_case")]
pub(crate) enum TonEmulationActionType {
    JettonSwap(TonEmulationJettonSwap),
    JettonTransfer(TonEmulationJettonTransfer),
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationTransaction {
    pub(crate) account: Address,
    pub(crate) description: Option<TransactionDescription>,
    pub(crate) account_state_before: TonEmulationAccountState,
    pub(crate) account_state_after: TonEmulationAccountState,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationAccountState {
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub(crate) balance: BigInt,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationJettonSwap {
    pub(crate) sender: Address,
    pub(crate) asset_in: Option<Address>,
    pub(crate) asset_out: Option<Address>,
    pub(crate) dex_incoming_transfer: TonEmulationJettonValue,
    pub(crate) dex_outgoing_transfer: TonEmulationJettonValue,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationJettonValue {
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub(crate) amount: BigInt,
}

#[derive(Deserialize)]
pub(crate) struct TonEmulationJettonTransfer {
    pub(crate) asset: Address,
    pub(crate) sender: Address,
    pub(crate) receiver: Address,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub(crate) amount: BigInt,
}
