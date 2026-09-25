use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotalFiatValue {
    pub value: f64,
    pub pnl_amount: f64,
    pub pnl_percentage: f64,
}
