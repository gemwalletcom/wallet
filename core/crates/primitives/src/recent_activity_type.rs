use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum RecentActivityType {
    Search,
    Transfer,
    Receive,
    FiatBuy,
    FiatSell,
    Swap,
    SwapSelect,
    Perpetual,
}
