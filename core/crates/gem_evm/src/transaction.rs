use crate::u256::u256_to_biguint;
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::Chain;
use primitives::swap::ApprovalData;

use crate::contracts::erc20::IERC20;
use crate::contracts::erc721::IERC721;
use crate::permit2::IAllowanceTransfer;
use crate::uniswap::deployment::is_permit2_contract_by_chain;

#[derive(Debug, Clone, PartialEq)]
pub enum EvmTransactionKind {
    Transfer,
    ContractCall,
    TokenApproval(ApprovalData),
}

pub fn decode_calldata(input: &str) -> Result<Vec<u8>, String> {
    let digits = input.strip_prefix("0x").or_else(|| input.strip_prefix("0X")).unwrap_or(input);
    primitives::hex::decode(digits).map_err(|error| error.to_string())
}

pub fn decode_transaction_kind(chain: Chain, to: &str, input: Option<&str>, value: Option<&BigUint>) -> Result<EvmTransactionKind, String> {
    let calldata = input.map(decode_calldata).transpose()?.unwrap_or_default();

    match decode_calldata_kind(chain, to, &calldata)? {
        EvmTransactionKind::TokenApproval(_) if value.is_some_and(|value| !value.is_zero()) => Ok(EvmTransactionKind::ContractCall),
        kind => Ok(kind),
    }
}

fn decode_calldata_kind(chain: Chain, to: &str, calldata: &[u8]) -> Result<EvmTransactionKind, String> {
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
        selector if selector.starts_with(&IERC20::increaseApprovalCall::SELECTOR) => {
            let approval = IERC20::increaseApprovalCall::abi_decode(calldata).map_err(|error| error.to_string())?;
            Ok(token_approval(to.to_string(), approval.spender, approval.addedValue))
        }
        selector if selector.starts_with(&IAllowanceTransfer::approveCall::SELECTOR) && is_permit2_contract_by_chain(&chain, to) => {
            let approval = IAllowanceTransfer::approveCall::abi_decode_validate(calldata).map_err(|error| error.to_string())?;
            Ok(token_approval(approval.token.to_string(), approval.spender, U256::from(approval.amount)))
        }
        _ => Ok(EvmTransactionKind::ContractCall),
    }
}

pub fn decode_nft_collection_approval_operator(calldata: &[u8]) -> Option<Address> {
    let approval = IERC721::setApprovalForAllCall::abi_decode(calldata).ok()?;
    approval.approved.then_some(approval.operator)
}

pub fn is_unlimited_approval(value: &BigUint) -> bool {
    value.bits() > 128
}

fn token_approval(token: String, spender: Address, value: U256) -> EvmTransactionKind {
    let value = u256_to_biguint(&value);
    EvmTransactionKind::TokenApproval(ApprovalData {
        token,
        spender: spender.to_string(),
        is_unlimited: is_unlimited_approval(&value),
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{U160, address, aliases::U48, hex};
    use primitives::contract_constants::{UNISWAP_PERMIT2_CONTRACT, ZKSYNC_UNISWAP_PERMIT2_CONTRACT};

    #[test]
    fn test_decode_calldata() {
        assert_eq!(decode_calldata("0xa9059cbb").unwrap(), vec![0xa9, 0x05, 0x9c, 0xbb]);
        assert_eq!(decode_calldata("0XA9059CBB").unwrap(), vec![0xa9, 0x05, 0x9c, 0xbb]);
        assert_eq!(decode_calldata("a9059cbb").unwrap(), vec![0xa9, 0x05, 0x9c, 0xbb]);
        assert_eq!(decode_calldata("0x").unwrap(), Vec::<u8>::new());
        assert!(decode_calldata("0x0xa9059cbb").is_err());
        assert!(decode_calldata("0xa9059cb").is_err());
        assert!(decode_calldata(" 0xa9059cbb").is_err());
    }

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

        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, None, None).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some(""), None).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some("0x"), Some(&one_coin)).unwrap(), EvmTransactionKind::Transfer);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some(approve), None).unwrap(), approval);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some(approve), Some(&BigUint::ZERO)).unwrap(), approval);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some(approve), Some(&BigUint::from(1u8))).unwrap(), EvmTransactionKind::ContractCall);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some(approve), Some(&one_coin)).unwrap(), EvmTransactionKind::ContractCall);
        assert_eq!(
            decode_transaction_kind(Chain::Ethereum, token, Some("0xa9059cbb000000000000000000000000111122223333444455556666777788889999aaaa"), None).unwrap(),
            EvmTransactionKind::ContractCall
        );
        assert_eq!(decode_transaction_kind(Chain::Ethereum, token, Some("0xdeadbeef"), None).unwrap(), EvmTransactionKind::ContractCall);
    }

    #[test]
    fn test_decode_nft_collection_approval_operator() {
        let operator = address!("0x2222222222222222222222222222222222222222");
        let approval = |approved: bool| IERC721::setApprovalForAllCall { operator, approved }.abi_encode();

        assert_eq!(decode_nft_collection_approval_operator(&approval(true)), Some(operator));
        assert_eq!(decode_nft_collection_approval_operator(&approval(false)), None);
        assert_eq!(decode_nft_collection_approval_operator(&approval(true)[..36]), None);
        assert_eq!(decode_nft_collection_approval_operator(&hex!("0x095ea7b3")), None);
    }

    #[test]
    fn test_is_unlimited_approval() {
        let power_of_two = |bits: u32| BigUint::from(1u8) << bits;

        assert!(is_unlimited_approval(&(power_of_two(256) - 1u8)));
        assert!(is_unlimited_approval(&(power_of_two(256) - 2u8)));
        assert!(is_unlimited_approval(&(power_of_two(160) - 1u8)));
        assert!(is_unlimited_approval(&(power_of_two(160) - 2u8)));
        assert!(is_unlimited_approval(&power_of_two(128)));
        assert!(!is_unlimited_approval(&(power_of_two(128) - 1u8)));
        assert!(!is_unlimited_approval(&BigUint::from(1_000_000u32)));
        assert!(!is_unlimited_approval(&BigUint::ZERO));
    }

    #[test]
    fn test_decode_unlimited_token_approval() {
        let token = "0x111122223333444455556666777788889999aaaa";
        let input = "0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbbffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

        assert_eq!(
            decode_transaction_kind(Chain::Ethereum, token, Some(input), None).unwrap(),
            EvmTransactionKind::TokenApproval(ApprovalData {
                token: token.to_string(),
                spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                value: u256_to_biguint(&U256::MAX),
                is_unlimited: true,
            })
        );
        for (value, is_unlimited) in [(U256::MAX - U256::from(1), true), ((U256::from(1) << 128) - U256::from(1), false)] {
            let input = format!("0x095ea7b300000000000000000000000022223333444455556666777788889999aaaabbbb{value:064x}");
            assert_eq!(
                decode_transaction_kind(Chain::Ethereum, token, Some(&input), None).unwrap(),
                EvmTransactionKind::TokenApproval(ApprovalData {
                    token: token.to_string(),
                    spender: "0x22223333444455556666777788889999aaAaBBbB".to_string(),
                    value: u256_to_biguint(&value),
                    is_unlimited,
                })
            );
        }
        assert!(decode_transaction_kind(Chain::Ethereum, token, Some("0x095ea7b3abcd"), None).is_err());
        assert!(decode_transaction_kind(Chain::Ethereum, token, Some("invalid"), None).is_err());
    }

    #[test]
    fn test_decode_allowance_approvals() {
        let token = address!("0x111122223333444455556666777788889999aaaa");
        let spender = address!("0x22223333444455556666777788889999aaaabbbb");
        let increases = [
            hex::encode_prefixed(IERC20::increaseAllowanceCall { spender, addedValue: U256::from(100) }.abi_encode()),
            hex::encode_prefixed(IERC20::increaseApprovalCall { spender, addedValue: U256::from(100) }.abi_encode()),
        ];
        let permit2 = hex::encode_prefixed(
            IAllowanceTransfer::approveCall {
                token,
                spender,
                amount: U160::MAX,
                expiration: U48::from(1_800_000_000u64),
            }
            .abi_encode(),
        );

        for increase in increases {
            assert_eq!(
                decode_transaction_kind(Chain::Ethereum, &token.to_string(), Some(&increase), None).unwrap(),
                EvmTransactionKind::TokenApproval(ApprovalData {
                    token: token.to_string(),
                    spender: spender.to_string(),
                    value: BigUint::from(100u64),
                    is_unlimited: false,
                })
            );
        }
        let permit2_approval = EvmTransactionKind::TokenApproval(ApprovalData {
            token: token.to_string(),
            spender: spender.to_string(),
            value: u256_to_biguint(&U256::from(U160::MAX)),
            is_unlimited: true,
        });
        assert_eq!(decode_transaction_kind(Chain::Ethereum, UNISWAP_PERMIT2_CONTRACT, Some(&permit2), None).unwrap(), permit2_approval);
        assert_eq!(decode_transaction_kind(Chain::ZkSync, ZKSYNC_UNISWAP_PERMIT2_CONTRACT, Some(&permit2), None).unwrap(), permit2_approval);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, &token.to_string(), Some(&permit2), None).unwrap(), EvmTransactionKind::ContractCall);
        assert_eq!(decode_transaction_kind(Chain::Ethereum, ZKSYNC_UNISWAP_PERMIT2_CONTRACT, Some(&permit2), None).unwrap(), EvmTransactionKind::ContractCall);
    }
}
