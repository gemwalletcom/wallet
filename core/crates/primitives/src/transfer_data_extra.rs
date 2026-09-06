use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::swap::ApprovalData;
use crate::{GasPriceType, TransactionType, TransferDataOutputAction, TransferDataOutputType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDataExtra {
    pub to: String,
    pub gas_limit: Option<BigInt>,
    pub gas_price: Option<GasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: TransferDataOutputType,
    pub output_action: TransferDataOutputAction,
    pub transaction_type: TransactionType,
    pub approval: Option<ApprovalData>,
}

impl TransferDataExtra {
    pub fn data_as_str(&self) -> Result<&str, &'static str> {
        let bytes = self.data.as_ref().ok_or("missing data")?;
        std::str::from_utf8(bytes).map_err(|_| "data is not valid utf8")
    }
}

impl Default for TransferDataExtra {
    fn default() -> Self {
        Self {
            to: "".to_string(),
            gas_limit: None,
            gas_price: None,
            data: None,
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Send,
            transaction_type: TransactionType::SmartContractCall,
            approval: None,
        }
    }
}
