use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{SolCall, sol};
use primitives::AssetId;

use crate::SwapperError;

sol! {
    interface IBridgers {
        function swap(address fromToken, string toToken, string destination, uint256 fromAmount, uint256 minReturnAmount) external;
        function swapEth(string toToken, string destination, uint256 minReturnAmount) external payable;
    }
}

pub(super) struct SwapCall {
    pub to_token: String,
    pub destination: String,
    pub min_return_amount: U256,
}

pub(super) fn get_swap_call(data: &[u8], value: U256, from_asset: &AssetId, from_amount: U256) -> Result<SwapCall, SwapperError> {
    match &from_asset.token_id {
        None => {
            let call = IBridgers::swapEthCall::abi_decode(data).map_err(|_| SwapperError::InvalidRoute)?;
            if value != from_amount {
                return Err(SwapperError::InvalidRoute);
            }
            Ok(SwapCall {
                to_token: call.toToken,
                destination: call.destination,
                min_return_amount: call.minReturnAmount,
            })
        }
        Some(token_id) => {
            let call = IBridgers::swapCall::abi_decode(data).map_err(|_| SwapperError::InvalidRoute)?;
            if call.fromToken != Address::from_str(token_id).map_err(|_| SwapperError::InvalidRoute)? || call.fromAmount != from_amount || value != U256::ZERO {
                return Err(SwapperError::InvalidRoute);
            }
            Ok(SwapCall {
                to_token: call.toToken,
                destination: call.destination,
                min_return_amount: call.minReturnAmount,
            })
        }
    }
}
