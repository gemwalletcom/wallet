use alloy_primitives::hex;
use alloy_sol_types::SolCall;
use primitives::TransactionType;

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
}
