use super::signer_mock::TEST_EVM_RECIPIENT;
use crate::{TransactionType, TransferDataExtra, TransferDataOutputAction, TransferDataOutputType};

impl TransferDataExtra {
    pub fn mock() -> Self {
        TransferDataExtra {
            to: TEST_EVM_RECIPIENT.to_string(),
            gas_limit: None,
            gas_price: None,
            data: None,
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Sign,
            transaction_type: TransactionType::SmartContractCall,
            approval: None,
        }
    }

    pub fn mock_encoded_transaction(data: Vec<u8>) -> Self {
        TransferDataExtra { data: Some(data), ..Self::mock() }
    }
}
