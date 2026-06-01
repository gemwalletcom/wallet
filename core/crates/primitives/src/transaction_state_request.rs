use crate::{Chain, SwapProvider, TransactionState, UInt64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: DateTime<Utc>,
    pub block_number: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionSwapStateRequest {
    pub transaction: TransactionStateRequest,
    pub state: TransactionState,
    pub swap_provider: SwapProvider,
    pub destination_chain: Chain,
}
