use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::{AssetId, Chain, TransactionState, TransactionType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionsFilter {
    pub asset_id: Option<AssetId>,
    pub chains: Vec<Chain>,
    pub transaction_types: Vec<TransactionType>,
    pub states: Vec<TransactionState>,
    pub asset_rank_greater_than: Option<i32>,
}
