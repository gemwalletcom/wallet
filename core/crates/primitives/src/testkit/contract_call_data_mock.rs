use super::signer_mock::TEST_EVM_RECIPIENT;
use crate::contract_call_data::ContractCallData;
use crate::{Asset, DelegationValidator, EarnType, TransactionInputType};

impl ContractCallData {
    pub fn mock() -> Self {
        ContractCallData {
            contract_address: TEST_EVM_RECIPIENT.to_string(),
            call_data: "abcd".to_string(),
            approval: None,
            gas_limit: None,
        }
    }

    pub fn mock_with_call_data(call_data: &str) -> Self {
        ContractCallData {
            call_data: call_data.to_string(),
            ..Self::mock()
        }
    }
}

impl TransactionInputType {
    pub fn mock_earn(data: ContractCallData) -> Self {
        Self::Earn {
            asset: Asset::mock_erc20(),
            earn_type: EarnType::Deposit(DelegationValidator::mock()),
            data,
        }
    }
}
