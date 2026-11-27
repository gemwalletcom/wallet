pub mod evm;
pub mod tron;

pub use evm::*;
use primitives::{Chain, swap::ApprovalData};
pub use tron::*;

use crate::{SwapperError, alien::RpcProvider, models::Permit2ApprovalData};
use alloy_primitives::U256;
use gem_evm::{permit2::IAllowanceTransfer, u256::u256_to_biguint};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub const DEFAULT_EVM_SWAP_GAS_LIMIT: u64 = 750_000;

pub const DEFAULT_TRON_SWAP_ENERGY_LIMIT: u64 = 130_000;

pub async fn check_token_approval(owner: String, token: String, spender: String, amount: U256, provider: Arc<dyn RpcProvider>, chain: Chain) -> Result<Option<ApprovalData>, SwapperError> {
    match chain {
        Chain::Tron => check_approval_trc20(owner, token, spender, amount, provider).await,
        _ => check_approval_erc20(owner, token, spender, amount, provider, &chain).await,
    }
}

pub(crate) fn permit2_approval(allowance: &IAllowanceTransfer::allowanceReturn, token: String, spender: String, permit2_contract: String, amount: U256) -> Result<Option<Permit2ApprovalData>, SwapperError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let expiration: u64 = allowance._1.try_into().map_err(|_| SwapperError::TransactionError("failed to convert expiration to u64".into()))?;

    if U256::from(allowance._0) < amount || expiration < timestamp {
        return Ok(Some(Permit2ApprovalData {
            token,
            spender,
            value: u256_to_biguint(&amount),
            permit2_contract,
            permit2_nonce: allowance._2.try_into().map_err(|_| SwapperError::TransactionError("failed to convert nonce to u64".into()))?,
        }));
    }

    Ok(None)
}

pub(crate) async fn join_permit2_approvals(
    approval: impl Future<Output = Result<Option<ApprovalData>, SwapperError>>,
    permit2: impl Future<Output = Result<Option<Permit2ApprovalData>, SwapperError>>,
    signed_permit: bool,
) -> Result<(Option<ApprovalData>, Option<Permit2ApprovalData>), SwapperError> {
    let permit2 = async {
        match signed_permit {
            true => Ok(None),
            false => permit2.await,
        }
    };
    futures::try_join!(approval, permit2)
}

/// Returns the swap transaction gas limit only when a separate approval transaction is required.
pub fn get_swap_gas_limit_with_approval(approval: &Option<ApprovalData>, swap_gas_limit: Option<String>, default_swap_gas_limit: u64) -> Option<String> {
    approval.as_ref().map(|_| swap_gas_limit.unwrap_or_else(|| default_swap_gas_limit.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_get_swap_gas_limit_with_approval() {
        let approval = Some(ApprovalData::make("0xtoken", "0xspender", BigUint::from(1000u64), true));

        assert_eq!(get_swap_gas_limit_with_approval(&approval, Some("250000".to_string()), DEFAULT_EVM_SWAP_GAS_LIMIT), Some("250000".to_string()));
        assert_eq!(get_swap_gas_limit_with_approval(&approval, None, DEFAULT_EVM_SWAP_GAS_LIMIT), Some("750000".to_string()));
        assert_eq!(get_swap_gas_limit_with_approval(&None, Some("250000".to_string()), DEFAULT_EVM_SWAP_GAS_LIMIT), None);
    }
}
