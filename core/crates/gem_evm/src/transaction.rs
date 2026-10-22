use crate::u256::u256_to_biguint;
use alloy_primitives::{Address, U160, U256, hex};
use alloy_sol_types::SolCall;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
use primitives::swap::ApprovalData;

use crate::contracts::erc20::IERC20;
use crate::permit2::IAllowanceTransfer;

#[derive(Debug, Clone, PartialEq)]
pub enum EvmTransactionKind {
    Transfer,
    ContractCall,
    TokenApproval(ApprovalData),
}

pub fn decode_transaction_kind(to: &str, input: Option<&str>, value: Option<&BigUint>) -> Result<EvmTransactionKind, String> {
    let calldata = input
        .map(|value| value.strip_prefix("0x").unwrap_or(value))
        .map(hex::decode)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();

    match decode_calldata_kind(to, &calldata)? {
        EvmTransactionKind::TokenApproval(_) if value.is_some_and(|value| !value.is_zero()) => Ok(EvmTransactionKind::ContractCall),
        kind => Ok(kind),
    }
}

fn decode_calldata_kind(to: &str, calldata: &[u8]) -> Result<EvmTransactionKind, String> {
    match calldata {
        [] => Ok(EvmTransactionKind::Transfer),
        selector if selector.starts_with(&IERC20::approveCall::SELECTOR) => {
            let approval = IERC20::approveCall::abi_decode(calldata).map_err(|error| error.to_string())?;
            Ok(token_approval(to.to_string(), approval.spender, approval.value))
        }
        selector if selector.starts_with(&IERC20::increaseAllowanceCall::SELECTOR) => {
            let approval = IERC20::increaseAllowanceCall::abi_decode(calldata).map_err(|error| error.to_string())?;
            Ok(token_approval(to.to_string(), approval.spender, approval.addedValue))
        }
        selector if selector.starts_with(&IAllowanceTransfer::approveCall::SELECTOR) && to.eq_ignore_ascii_case(UNISWAP_PERMIT2_CONTRACT) => {
            let approval = IAllowanceTransfer::approveCall::abi_decode_validate(calldata).map_err(|error| error.to_string())?;
            Ok(token_approval(approval.token.to_string(), approval.spender, U256::from(approval.amount)))
        }
        _ => Ok(EvmTransactionKind::ContractCall),
    }
}

fn token_approval(token: String, spender: Address, value: U256) -> EvmTransactionKind {
    EvmTransactionKind::TokenApproval(ApprovalData {
        token,
        spender: spender.to_string(),
        value: u256_to_biguint(&value),
        is_unlimited: value == U256::MAX || value == U256::from(U160::MAX),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, aliases::U48};

    #[test]
    fn test_decode_transaction_kind() {
        let token = "0x111122223333444455556666777788889999aaaa";
        let approve = "0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbb0000000000000000000000000000000000000000000000000000000000000064";
        let approval = EvmTransactionKind::TokenApproval(ApprovalData {
            token: token.to_string(),
            spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
            value: BigUint::from(100u64),
            is_unlimited: false,
        });
        let one_coin = BigUint::from(10_000_000_000_000_000u64);

        assert_eq!(decode_transaction_kind(token, None, None).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(token, Some(""), None).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(token, Some("0x"), Some(&one_coin)).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(token, Some(approve), None).unwrap(), approval);
        assert_eq!(decode_transaction_kind(token, Some(approve), Some(&BigUint::ZERO)).unwrap(), approval);
        assert_eq!(decode_transaction_kind(token, Some(approve), Some(&BigUint::from(1u8))).unwrap(), EvmTransactionKind::ContractCall);
        assert_eq!(decode_transaction_kind(token, Some(approve), Some(&one_coin)).unwrap(), EvmTransactionKind::ContractCall);
        assert_eq!(
            decode_transaction_kind(token, Some("0xa9059cbb000000000000000000000000111122223333444455556666777788889999aaaa"), None).unwrap(),
            EvmTransactionKind::ContractCall
        );
        assert_eq!(decode_transaction_kind(token, Some("0xdeadbeef"), None).unwrap(), EvmTransactionKind::ContractCall);
    }

    #[test]
    fn test_decode_unlimited_token_approval() {
        let token = "0x111122223333444455556666777788889999aaaa";
        let input = "0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbbffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(
            decode_transaction_kind(token, Some(input), None).unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                value: u256_to_biguint(&U256::MAX),
                is_unlimited: true,
            })
        );
        assert!(decode_transaction_kind(token, Some("0x095ea7b3abcd"), None).is_err());
        assert!(decode_transaction_kind(token, Some("invalid"), None).is_err());
    }

    #[test]
    fn test_decode_allowance_approvals() {
        let token = address!("0x111122223333444455556666777788889999aaaa");
        let spender = address!("0x22223333444455556666777788889999aaaabbbb");
        let increase = hex::encode_prefixed(IERC20::increaseAllowanceCall { spender, addedValue: U256::from(100) }.abi_encode());
        let permit2 = hex::encode_prefixed(
            IAllowanceTransfer::approveCall {
                token,
                spender,
                amount: U160::MAX,
                expiration: U48::from(1_800_000_000u64),
            }
            .abi_encode(),
        );

        assert_eq!(
            decode_transaction_kind(&token.to_string(), Some(&increase), None).unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: spender.to_string(),
                value: BigUint::from(100u64),
                is_unlimited: false,
            })
        );
        assert_eq!(
            decode_transaction_kind(UNISWAP_PERMIT2_CONTRACT, Some(&permit2), None).unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: spender.to_string(),
                value: u256_to_biguint(&U256::from(U160::MAX)),
                is_unlimited: true,
            })
        );
        assert_eq!(decode_transaction_kind(&token.to_string(), Some(&permit2), None).unwrap(), EvmTransactionKind::ContractCall);
    }
}
