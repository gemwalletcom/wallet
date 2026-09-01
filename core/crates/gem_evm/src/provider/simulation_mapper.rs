use std::collections::HashMap;

use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationWarning};

use crate::ethereum_address_checksum;
use crate::provider::transfer_decoder::decode_transfer_action;
use crate::rpc::model::TraceCallResult;

pub fn map_simulation_result(chain: Chain, signer: &str, trace: &TraceCallResult) -> SimulationResult {
    if let Some(error) = trace.root_call_error() {
        return SimulationResult::new(vec![SimulationWarning::execution_error(error)], vec![]);
    }

    SimulationResult {
        balance_changes: map_balance_changes(chain, signer, trace),
        ..Default::default()
    }
}

fn map_balance_changes(chain: Chain, signer: &str, trace: &TraceCallResult) -> Vec<SimulationBalanceChange> {
    let mut changes = Vec::new();

    if let Some(diff) = native_balance_diff(signer, trace)
        && !diff.is_zero()
    {
        changes.push(SimulationBalanceChange::new(AssetId::from_chain(chain), diff));
    }

    let mut token_deltas: HashMap<&str, BigInt> = HashMap::new();
    for entry in &trace.trace {
        if entry.error.is_some() || entry.action.call_type.as_deref() != Some("call") {
            continue;
        }
        let Some(token) = entry.action.to.as_deref() else { continue };
        let Some(transfer) = decode_transfer_action(&entry.action) else { continue };

        if transfer.from.eq_ignore_ascii_case(signer) {
            *token_deltas.entry(token).or_default() -= &transfer.amount;
        }
        if transfer.to.eq_ignore_ascii_case(signer) {
            *token_deltas.entry(token).or_default() += &transfer.amount;
        }
    }

    for (token, delta) in token_deltas {
        if !delta.is_zero()
            && let Ok(token_address) = ethereum_address_checksum(token)
        {
            changes.push(SimulationBalanceChange::new(AssetId::from_token(chain, &token_address), delta));
        }
    }

    changes.sort_by_key(|change| change.asset_id.to_string());
    changes
}

fn native_balance_diff(signer: &str, trace: &TraceCallResult) -> Option<BigInt> {
    let (_, state) = trace.state_diff.iter().find(|(address, _)| address.eq_ignore_ascii_case(signer))?;
    let (from, to) = state.balance_change()?;
    Some(to - from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U160, U256, address};
    use alloy_sol_types::SolCall;
    use primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID;
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
    use primitives::testkit::json_rpc::load_json_rpc_result;
    use primitives::testkit::signer_mock::TEST_EVM_RECIPIENT;
    use std::str::FromStr;

    use crate::contracts::IERC20;
    use crate::permit2::IAllowanceTransfer;
    use crate::rpc::model::{TraceCallAction, TraceCallEntry};
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_map_simulation_result_native_transfer() {
        let trace: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_native_transfer.json"));

        let result = map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &trace);

        assert!(result.warnings.is_empty());
        assert_eq!(
            result.balance_changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                value: BigInt::from(-10000000000000000i64),
                decimals: 0,
                name: None,
                symbol: None,
            }]
        );
    }

    #[test]
    fn test_map_simulation_result_root_revert_returns_validation_warning() {
        let reverted_root: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_reverted_root.json"));
        let result = map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &reverted_root);

        assert_eq!(result.warnings, vec![SimulationWarning::execution_error("Reverted")]);
        assert!(result.balance_changes.is_empty());

        let reverted_subcall_only: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_subcall_reverted.json"));
        assert!(map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &reverted_subcall_only).warnings.is_empty());
    }

    #[test]
    fn test_map_simulation_result_erc20_transfer_ignores_delegatecall_duplicate() {
        let trace: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_erc20_transfer_proxy.json"));

        assert_eq!(
            map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &trace).balance_changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                value: BigInt::from(-1000000i64),
                decimals: 0,
                name: None,
                symbol: None,
            }]
        );
    }

    #[test]
    fn test_map_simulation_result_permit2_relies_on_inner_erc20_transfer_from() {
        let permit2_call = TraceCallEntry {
            action: TraceCallAction::mock(
                UNISWAP_PERMIT2_CONTRACT,
                IAllowanceTransfer::transferFromCall {
                    from: Address::from_str(TEST_ADDRESS).unwrap(),
                    to: Address::from_str(TEST_EVM_RECIPIENT).unwrap(),
                    amount: U160::from(1_000_000u64),
                    token: address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                }
                .abi_encode(),
            ),
            error: None,
            trace_address: vec![0],
        };
        let token_transfer_call = TraceCallEntry {
            action: TraceCallAction::mock(
                ETHEREUM_USDC_TOKEN_ID,
                IERC20::transferFromCall {
                    from: Address::from_str(TEST_ADDRESS).unwrap(),
                    to: Address::from_str(TEST_EVM_RECIPIENT).unwrap(),
                    value: U256::from(1_000_000u64),
                }
                .abi_encode(),
            ),
            error: None,
            trace_address: vec![0, 0],
        };
        let trace = TraceCallResult {
            state_diff: HashMap::new(),
            trace: vec![permit2_call, token_transfer_call],
        };

        assert_eq!(
            map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &trace).balance_changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
                value: BigInt::from(-1000000i64),
                decimals: 0,
                name: None,
                symbol: None,
            }]
        );
    }

    #[test]
    fn test_map_simulation_result_skips_reverted_subcall() {
        let trace: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_reverted_transfer.json"));

        assert_eq!(map_simulation_result(Chain::Ethereum, TEST_ADDRESS, &trace).balance_changes, vec![]);
    }
}
