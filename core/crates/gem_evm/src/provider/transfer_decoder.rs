use alloy_primitives::{Address, U256};
use alloy_sol_types::SolInterface;
use num_bigint::BigInt;
use primitives::hex;

use crate::contracts::eip3009::IEIP3009::IEIP3009Calls;
use crate::contracts::erc20::IERC20::IERC20Calls;
use crate::rpc::model::TraceCallAction;
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

pub fn decode_transfer_action(action: &TraceCallAction) -> Option<DecodedTransfer> {
    let calldata = hex::decode_hex(&action.input).ok()?;

    if let Ok(call) = IERC20Calls::abi_decode(&calldata) {
        return match call {
            IERC20Calls::transfer(call) => Some(DecodedTransfer::new(action.from.clone(), call.to, &call.value)),
            IERC20Calls::transferFrom(call) => Some(DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
            _ => None,
        };
    }

    if let Ok(call) = IEIP3009Calls::abi_decode(&calldata) {
        return match call {
            IEIP3009Calls::transferWithAuthorization(call) => Some(DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
            IEIP3009Calls::receiveWithAuthorization(call) => Some(DecodedTransfer::new(call.from.to_string(), call.to, &call.value)),
        };
    }

    None
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
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_decode_transfer_action_eip3009() {
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

        let transfer = decode_transfer_action(&TraceCallAction::mock(ETHEREUM_USDC_TOKEN_ID, calldata)).unwrap();
        assert_eq!(transfer.amount, BigInt::from(1_000_000));
    }
}
