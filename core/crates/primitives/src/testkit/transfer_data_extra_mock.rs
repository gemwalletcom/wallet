use super::signer_mock::TEST_EVM_RECIPIENT;
use crate::swap::ApprovalData;
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

    pub fn mock_signature(data: Vec<u8>, approval: Option<ApprovalData>) -> Self {
        TransferDataExtra {
            data: Some(data),
            output_type: TransferDataOutputType::Signature,
            transaction_type: TransactionType::Transfer,
            approval,
            ..Self::mock()
        }
    }
}
