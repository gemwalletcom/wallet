use std::str::FromStr;

use alloy_primitives::{Address, U256, hex};
use alloy_sol_types::{SolCall, sol};

use primitives::AssetId;

use super::model::{EvmTransaction, SwapRequest, get_to_token};
use crate::SwapperError;

sol! {
    interface IBridgers {
        function swap(address fromToken, string toToken, string destination, uint256 fromAmount, uint256 minReturnAmount) external;
        function swapEth(string toToken, string destination, uint256 minReturnAmount) external payable;
    }
}

struct SwapCall {
    to_token: String,
    destination: String,
    min_return_amount: U256,
}

pub(super) fn get_transaction_value(transaction: &EvmTransaction, router: &str, from_asset: &AssetId, to_code: &str, swap: &SwapRequest) -> Result<U256, SwapperError> {
    if Address::from_str(&transaction.to).map_err(|_| SwapperError::InvalidRoute)? != Address::from_str(router)? {
        return Err(SwapperError::InvalidRoute);
    }
    let value = U256::from_str(&transaction.value).map_err(|_| SwapperError::InvalidRoute)?;
    let data = hex::decode(&transaction.data).map_err(|_| SwapperError::InvalidRoute)?;
    let call = get_swap_call(&data, value, from_asset, U256::from_str(&swap.quote.from_token_amount)?)?;
    if call.destination != swap.to_address || call.min_return_amount != U256::from_str(&swap.amount_out_min)? || call.to_token != get_to_token(to_code, &swap.slippage) {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(value)
}

fn get_swap_call(data: &[u8], value: U256, from_asset: &AssetId, from_amount: U256) -> Result<SwapCall, SwapperError> {
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

#[cfg(test)]
mod tests {
    use primitives::{Chain, asset_constants::BASE_USDC_ASSET_ID};

    use super::*;
    use crate::bridgers::{
        model::{BridgersResponse, QuoteRequest, SwapData},
        testkit::TEST_EVM_WALLET,
    };

    #[test]
    fn test_get_transaction_value() {
        let native: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_opbnb_native.json")).unwrap();
        let token: BridgersResponse = serde_json::from_str(include_str!("testdata/swap_base_usdc_bsc_usdt.json")).unwrap();
        let native_tx = serde_json::from_value::<SwapData>(native.data).unwrap().tx_data;
        let token_tx = serde_json::from_value::<SwapData>(token.data).unwrap().tx_data;
        let bnb = AssetId::from_chain(Chain::OpBNB);
        let native_swap = SwapRequest {
            quote: QuoteRequest {
                from_token_amount: "100000000000000000".to_string(),
                ..Default::default()
            },
            to_address: TEST_EVM_WALLET.to_string(),
            amount_out_min: "77563731000000000000".to_string(),
            slippage: "0.005".to_string(),
            ..Default::default()
        };
        let token_swap = SwapRequest {
            quote: QuoteRequest {
                from_token_amount: "100000000".to_string(),
                ..Default::default()
            },
            amount_out_min: "99213808000000000000".to_string(),
            ..native_swap.clone()
        };
        let other_destination = SwapRequest {
            to_address: "0x0000000000000000000000000000000000000001".to_string(),
            ..native_swap.clone()
        };
        let opbnb_router = "0x8F957Ed3F969D7b6e5d6dF81e61A5Ff45F594dD1";

        assert_eq!(get_transaction_value(&native_tx, opbnb_router, &bnb, "USDT(BSC)", &native_swap).unwrap(), U256::from(100_000_000_000_000_000u64));
        assert_eq!(
            get_transaction_value(&token_tx, "0xa18968Cc31232724F1DBD0D1e8d0b323D89F3501", &BASE_USDC_ASSET_ID, "USDT(BSC)", &token_swap).unwrap(),
            U256::ZERO
        );
        assert_eq!(get_transaction_value(&native_tx, opbnb_router, &bnb, "USDT(BSC)", &other_destination).unwrap_err(), SwapperError::InvalidRoute);
    }
}
