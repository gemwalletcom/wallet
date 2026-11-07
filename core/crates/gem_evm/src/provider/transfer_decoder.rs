use alloy_primitives::{Address, U256};
use alloy_sol_types::{SolCall, SolInterface};
use num_bigint::BigInt;
use primitives::hex;

use crate::contracts::eip3009::IEIP3009::IEIP3009Calls;
use crate::contracts::erc20::IERC20::{self, IERC20Calls};
use crate::contracts::erc677::IERC677;
use crate::rpc::model::TraceCallEntry;
use crate::u256::u256_to_biguint;

pub struct DecodedTransfer {
    pub from: String,
    pub to: String,
    pub amount: BigInt,
}

impl DecodedTransfer {
    fn new(from: String, to: Address, value: &U256) -> Self {
        Self {
            from,
            to: to.to_string(),
            amount: BigInt::from(u256_to_biguint(value)),
        }
    }
}

pub fn decode_transfer(entry: &TraceCallEntry) -> Option<DecodedTransfer> {
    let action = &entry.action;
    let calldata = hex::decode_hex(&action.input).ok()?;
    let output = entry.result.as_ref().and_then(|result| hex::decode_hex(&result.output).ok()).unwrap_or_default();

    if let Ok(call) = IERC20Calls::abi_decode(&calldata) {
        return match call {
            IERC20Calls::transfer(call) => accepted::<IERC20::transferCall>(&output).then(|| DecodedTransfer::new(action.from.clone(), call.to, &call.value)),
            IERC20Calls::transferFrom(call) => accepted::<IERC20::transferFromCall>(&output).then(|| DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
            _ => None,
        };
    }

    if let Ok(call) = IEIP3009Calls::abi_decode(&calldata) {
        return match call {
            IEIP3009Calls::transferWithAuthorization(call) => Some(DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
            IEIP3009Calls::receiveWithAuthorization(call) => Some(DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
        };
    }

    if let Ok(call) = IERC677::transferAndCallCall::abi_decode(&calldata) {
        return accepted::<IERC677::transferAndCallCall>(&output).then(|| DecodedTransfer::new(action.from.clone(), call.to, &call.value));
    }

    None
}

fn accepted<C: SolCall<Return = bool>>(output: &[u8]) -> bool {
    output.is_empty() || C::abi_decode_returns(output).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use alloy_sol_types::SolCall;
    use primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use std::str::FromStr;

    use crate::contracts::IEIP3009;
    use crate::rpc::model::TraceCallAction;
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_decode_transfer_eip3009() {
        let calldata = IEIP3009::transferWithAuthorizationCall {
            from: Address::from_str(TEST_ADDRESS).unwrap(),
            to: Address::from_str(TEST_EVM_RECIPIENT).unwrap(),
            value: U256::from(1_000_000u64),
            validAfter: U256::from(0u64),
            validBefore: U256::from(9_999_999_999u64),
            nonce: B256::ZERO,
            v: 27,
            r: B256::ZERO,
            s: B256::ZERO,
        }
        .abi_encode();

        let entry = TraceCallEntry {
            action: TraceCallAction::mock(ETHEREUM_USDC_TOKEN_ID, calldata),
            error: None,
            result: None,
            trace_address: vec![],
        };
        let transfer = decode_transfer(&entry).unwrap();
        assert_eq!(transfer.amount, BigInt::from(1_000_000));
    }
}
