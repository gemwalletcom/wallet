use std::str::FromStr;

use alloy_primitives::{Address, U256, hex};
use alloy_sol_types::{SolCall, sol};

use super::{
    asset::Token,
    model::{EvmTransaction, SwapRequest, get_to_token},
};
use crate::SwapperError;

sol! {
    interface IBridgers {
        function swap(address fromToken, string toToken, string destination, uint256 fromAmount, uint256 minReturnAmount) external;
        function swapEth(string toToken, string destination, uint256 minReturnAmount) external payable;
    }
}

pub(super) fn get_transaction_value(transaction: &EvmTransaction, router: &str, from_token: &Token, to_token: &Token, swap: &SwapRequest) -> Result<U256, SwapperError> {
    if Address::from_str(&transaction.to).map_err(|_| SwapperError::InvalidRoute)? != Address::from_str(router)? {
        return Err(SwapperError::InvalidRoute);
    }
    let from_amount = U256::from_str(&swap.quote.from_token_amount)?;
    let value = U256::from_str(&transaction.value).map_err(|_| SwapperError::InvalidRoute)?;
    let data = hex::decode(&transaction.data).map_err(|_| SwapperError::InvalidRoute)?;
    let (call_to_token, destination, min_return_amount) = match &from_token.asset_id.token_id {
        None => {
            let call = IBridgers::swapEthCall::abi_decode(&data).map_err(|_| SwapperError::InvalidRoute)?;
            if value != from_amount {
                return Err(SwapperError::InvalidRoute);
            }
            (call.toToken, call.destination, call.minReturnAmount)
        }
        Some(token_id) => {
            let call = IBridgers::swapCall::abi_decode(&data).map_err(|_| SwapperError::InvalidRoute)?;
            if call.fromToken != Address::from_str(token_id).map_err(|_| SwapperError::InvalidRoute)? || call.fromAmount != from_amount || value != U256::ZERO {
                return Err(SwapperError::InvalidRoute);
            }
            (call.toToken, call.destination, call.minReturnAmount)
        }
    };
    if destination != swap.to_address || min_return_amount != U256::from_str(&swap.amount_out_min)? || call_to_token != get_to_token(to_token.code, &swap.slippage) {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use primitives::{
        AssetId, Chain,
        asset_constants::{BASE_USDC_ASSET_ID, SMARTCHAIN_USDT_ASSET_ID},
    };

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
        let bnb = Token::from_asset_id(&AssetId::from_chain(Chain::OpBNB)).unwrap();
        let base_usdc = Token::from_asset_id(&BASE_USDC_ASSET_ID).unwrap();
        let bsc_usdt = Token::from_asset_id(&SMARTCHAIN_USDT_ASSET_ID).unwrap();
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

        assert_eq!(get_transaction_value(&native_tx, opbnb_router, &bnb, &bsc_usdt, &native_swap).unwrap(), U256::from(100_000_000_000_000_000u64));
        assert_eq!(get_transaction_value(&token_tx, "0xa18968Cc31232724F1DBD0D1e8d0b323D89F3501", &base_usdc, &bsc_usdt, &token_swap).unwrap(), U256::ZERO);
        assert_eq!(get_transaction_value(&native_tx, opbnb_router, &bnb, &bsc_usdt, &other_destination).unwrap_err(), SwapperError::InvalidRoute);
    }
}
