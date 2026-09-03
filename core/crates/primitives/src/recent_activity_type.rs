use serde::{Deserialize, Serialize};
use strum::EnumIter;
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
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
