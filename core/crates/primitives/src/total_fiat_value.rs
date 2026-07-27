use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TotalFiatValue {
    pub value: f64,
    pub pnl_amount: f64,
    pub pnl_percentage: f64,
}
