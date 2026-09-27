use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Model)]
#[serde(rename_all = "camelCase")]
#[model(swift = "Equatable, Sendable")]
pub enum TpslType {
    TakeProfit,
    StopLoss,
}
