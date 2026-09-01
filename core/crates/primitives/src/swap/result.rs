use super::SwapStatus;
use crate::TransactionSwapMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub status: SwapStatus,
    pub metadata: Option<TransactionSwapMetadata>,
    pub eta_in_seconds: Option<u32>,
}
