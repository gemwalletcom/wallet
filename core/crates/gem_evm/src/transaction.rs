use alloy_primitives::{U160, U256, hex};
use alloy_sol_types::SolCall;
use primitives::swap::ApprovalData;

use crate::contracts::erc20::IERC20;

#[derive(Debug, Clone, PartialEq)]
pub enum EvmTransactionKind {
    Transfer,
    ContractCall,
    TokenApproval(ApprovalData),
}

pub fn decode_transaction_kind(token: &str, input: Option<&str>) -> Result<EvmTransactionKind, String> {
    let calldata = input
        .map(|value| value.strip_prefix("0x").unwrap_or(value))
        .map(hex::decode)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();

    match calldata.as_slice() {
        [] => Ok(EvmTransactionKind::Transfer),
        selector if selector.starts_with(&IERC20::approveCall::SELECTOR) => {
            let approval = IERC20::approveCall::abi_decode(&calldata).map_err(|error| error.to_string())?;
            Ok(EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: approval.spender.to_string(),
                value: approval.value.to_string(),
                is_unlimited: approval.value == U256::MAX || approval.value == U256::from(U160::MAX),
            }))
        }
        _ => Ok(EvmTransactionKind::ContractCall),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction_kind() {
        let token = "0x111122223333444455556666777788889999aaaa";
        assert_eq!(decode_transaction_kind(token, None).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(token, Some("")).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(token, Some("0x")).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(
            decode_transaction_kind(
                token,
                Some("0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbb0000000000000000000000000000000000000000000000000000000000000064")
            )
            .unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                value: "100".to_string(),
                is_unlimited: false,
            })
        );
        assert_eq!(
            decode_transaction_kind(token, Some("0xa9059cbb000000000000000000000000111122223333444455556666777788889999aaaa")).unwrap(),
            EvmTransactionKind::ContractCall
        );
        assert_eq!(decode_transaction_kind(token, Some("0xdeadbeef")).unwrap(), EvmTransactionKind::ContractCall);
    }

    #[test]
    fn test_decode_unlimited_token_approval() {
        let token = "0x111122223333444455556666777788889999aaaa";
        let input = "0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbbffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(
            decode_transaction_kind(token, Some(input)).unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                value: U256::MAX.to_string(),
                is_unlimited: true,
            })
        );
        assert!(decode_transaction_kind(token, Some("0x095ea7b3abcd")).is_err());
        assert!(decode_transaction_kind(token, Some("invalid")).is_err());
    }
}
