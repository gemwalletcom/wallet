use alloy_primitives::{U160, U256, hex};
use alloy_sol_types::SolCall;
use primitives::{TransactionType, swap::ApprovalData};

use crate::contracts::erc20::IERC20;

pub fn decode_transaction_type(input: Option<&str>) -> TransactionType {
    let calldata = input
        .map(|value| value.strip_prefix("0x").unwrap_or(value))
        .and_then(|value| hex::decode(value).ok())
        .unwrap_or_default();
    match calldata.as_slice() {
        [] => TransactionType::Transfer,
        selector if selector.starts_with(&IERC20::approveCall::SELECTOR) => TransactionType::TokenApproval,
        _ => TransactionType::SmartContractCall,
    }
}

pub fn decode_approval_data(token: &str, input: Option<&str>) -> Result<Option<ApprovalData>, String> {
    let Some(input) = input else {
        return Ok(None);
    };
    let calldata = hex::decode(input.strip_prefix("0x").unwrap_or(input)).map_err(|error| error.to_string())?;
    if !calldata.starts_with(&IERC20::approveCall::SELECTOR) {
        return Ok(None);
    }

    let approval = IERC20::approveCall::abi_decode(&calldata).map_err(|error| error.to_string())?;
    Ok(Some(ApprovalData {
        token: token.to_string(),
        spender: approval.spender.to_string(),
        value: approval.value.to_string(),
        is_unlimited: approval.value == U256::MAX || approval.value == U256::from(U160::MAX),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction_type() {
        assert_eq!(decode_transaction_type(None), TransactionType::Transfer);
        assert_eq!(decode_transaction_type(Some("")), TransactionType::Transfer);
        assert_eq!(decode_transaction_type(Some("0x")), TransactionType::Transfer);
        assert_eq!(
            decode_transaction_type(Some("0x095ea7b3000000000000000000000000111122223333444455556666777788889999aaaa")),
            TransactionType::TokenApproval
        );
        assert_eq!(
            decode_transaction_type(Some("0x095EA7B3000000000000000000000000111122223333444455556666777788889999AAAA")),
            TransactionType::TokenApproval
        );
        assert_eq!(
            decode_transaction_type(Some("0xa9059cbb000000000000000000000000111122223333444455556666777788889999aaaa")),
            TransactionType::SmartContractCall
        );
        assert_eq!(decode_transaction_type(Some("0xdeadbeef")), TransactionType::SmartContractCall);
    }

    #[test]
    fn test_decode_approval_data() {
        let token = "0x111122223333444455556666777788889999aaaa";
        let input = "0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbbffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(
            decode_approval_data(token, Some(input)).unwrap(),
            Some(ApprovalData {
                token: token.to_string(),
                spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                value: U256::MAX.to_string(),
                is_unlimited: true,
            })
        );
        assert_eq!(decode_approval_data(token, Some("0xdeadbeef")).unwrap(), None);
        assert!(decode_approval_data(token, Some("0x095ea7b3abcd")).is_err());
    }
}
