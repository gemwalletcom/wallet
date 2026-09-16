use alloy_sol_types::SolCall;
use gem_evm::permit2::IAllowanceTransfer;
use num_bigint::BigUint;
use primitives::{Address as _, ApprovalData, SignerError};

use crate::address::TronAddress;
use crate::trc20::{TRC20_APPROVE_SELECTOR, decode_approval};

pub(crate) const PERMIT2_APPROVE_SELECTOR: [u8; 4] = IAllowanceTransfer::approveCall::SELECTOR;
const SUN_PERMIT2_CONTRACT: &str = "TTJxU3P8rHycAyFY4kVtGNfmnMH4ezcuM9";

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionApproval {
    pub approval: ApprovalData,
    pub contract: String,
    pub expiration: Option<u64>,
}

impl TransactionApproval {
    pub(crate) fn decode(contract: TronAddress, data: &[u8]) -> Result<Option<Self>, SignerError> {
        let contract = contract.encode();
        if data.starts_with(&TRC20_APPROVE_SELECTOR) {
            let approval = decode_approval(data).ok_or_else(|| SignerError::invalid_input("Invalid TRC20 approval calldata"))?;
            return Ok(Some(Self {
                approval: ApprovalData {
                    token: contract.clone(),
                    spender: approval.spender.encode(),
                    is_unlimited: approval.value == BigUint::from_bytes_be(&[0xff; 32]),
                    value: approval.value,
                },
                contract,
                expiration: None,
            }));
        }
        if data.starts_with(&PERMIT2_APPROVE_SELECTOR) && contract == SUN_PERMIT2_CONTRACT {
            return Self::decode_permit2(contract, data)
                .map(Some)
                .ok_or_else(|| SignerError::invalid_input("Invalid Permit2 approval calldata"));
        }
        Ok(None)
    }

    fn decode_permit2(contract: String, data: &[u8]) -> Option<Self> {
        let approval = IAllowanceTransfer::approveCall::abi_decode_validate(data).ok()?;
        if data.len() != PERMIT2_APPROVE_SELECTOR.len() + approval.abi_encoded_size() {
            return None;
        }
        let value = approval.amount.to_be_bytes::<20>();
        Some(Self {
            approval: ApprovalData {
                token: TronAddress::from(approval.token.into_array()).encode(),
                spender: TronAddress::from(approval.spender.into_array()).encode(),
                value: BigUint::from_bytes_be(&value),
                is_unlimited: value == [0xff; 20],
            },
            contract,
            expiration: Some(approval.expiration.to::<u64>()),
        })
    }
}
