use alloy_primitives::{Address, U256};
use alloy_sol_types::{SolCall, SolInterface};
use num_bigint::BigInt;
use primitives::hex;

use crate::contracts::dai::IDai::IDaiCalls;
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

    if let Ok(call) = IDaiCalls::abi_decode(&calldata) {
        return Some(match call {
            IDaiCalls::push(call) => DecodedTransfer::new(action.from.clone(), call.usr, &call.wad),
            IDaiCalls::pull(call) => DecodedTransfer {
                from: call.usr.to_string(),
                to: action.from.clone(),
                amount: BigInt::from(u256_to_biguint(&call.wad)),
            },
            IDaiCalls::r#move(call) => DecodedTransfer::new(call.src.to_string(), call.dst, &call.wad),
        });
    }

    if let Ok(call) = IERC677::transferAndCallCall::abi_decode(&calldata) {
        return accepted::<IERC677::transferAndCallCall>(&output).then(|| DecodedTransfer::new(action.from.clone(), call.to, &call.value));
    }

    None
}

fn accepted<C: SolCall<Return = bool>>(output: &[u8]) -> bool {
    C::abi_decode_returns(output).unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use alloy_sol_types::SolCall;
    use primitives::asset_constants::{ETHEREUM_DAI_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID};
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use std::str::FromStr;

    use crate::contracts::{IDai, IEIP3009};
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

    #[test]
    fn test_decode_transfer_dai_pull_and_move() {
        let entry = |calldata: Vec<u8>| TraceCallEntry {
            action: TraceCallAction::mock(ETHEREUM_DAI_TOKEN_ID, calldata),
            error: None,
            result: None,
            trace_address: vec![],
        };
        let recipient = Address::from_str(TEST_EVM_RECIPIENT).unwrap();

        let pull = decode_transfer(&entry(IDai::pullCall { usr: recipient, wad: U256::from(5u64) }.abi_encode())).unwrap();
        assert_eq!((pull.from, pull.to), (recipient.to_string(), TEST_ADDRESS.to_string()));

        let moved = decode_transfer(&entry(
            IDai::moveCall {
                src: Address::from_str(TEST_ADDRESS).unwrap(),
                dst: recipient,
                wad: U256::from(5u64),
            }
            .abi_encode(),
        ))
        .unwrap();
        assert_eq!((moved.from, moved.to), (TEST_ADDRESS.to_string(), recipient.to_string()));
    }
}
