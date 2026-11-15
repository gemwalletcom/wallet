use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct XrpStepData {
    pub to: String,
    pub currency: String,
    pub amount: String,
    pub memo: String,
}
