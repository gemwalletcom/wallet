use std::str::FromStr;

use alloy_primitives::{Address, U256, hex};
use gem_evm::u256::biguint_to_u256;

use primitives::AssetId;

use super::{
    contracts::get_swap_call,
    model::{EvmTransaction, SwapRequest},
};
use crate::SwapperError;

pub(super) fn get_transaction_value(transaction: &EvmTransaction, router: &str, from_asset: &AssetId, to_code: &str, swap: &SwapRequest) -> Result<U256, SwapperError> {
    if Address::from_str(&transaction.to).map_err(|_| SwapperError::InvalidRoute)? != Address::from_str(router)? {
        return Err(SwapperError::InvalidRoute);
    }
    let value = U256::from_str(&transaction.value).map_err(|_| SwapperError::InvalidRoute)?;
    let data = hex::decode(&transaction.data).map_err(|_| SwapperError::InvalidRoute)?;
    let from_amount = biguint_to_u256(&swap.quote.from_token_amount).ok_or(SwapperError::InvalidRoute)?;
    let min_return_amount = biguint_to_u256(&swap.amount_out_min).ok_or(SwapperError::InvalidRoute)?;
    let call = get_swap_call(&data, value, from_asset, from_amount)?;
    if call.destination != swap.to_address || call.min_return_amount != min_return_amount || !is_valid_to_token(&call.to_token, to_code, &swap.slippage) {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(value)
}

fn is_valid_to_token(to_token: &str, code: &str, slippage: &str) -> bool {
    match to_token.split('|').collect::<Vec<_>>().as_slice() {
        [token, _channel, token_slippage, ..] => *token == code && *token_slippage == slippage,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
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
                from_token_amount: BigUint::from(100_000_000_000_000_000u64),
                ..Default::default()
            },
            to_address: TEST_EVM_WALLET.to_string(),
            amount_out_min: BigUint::from(77_563_731_000_000_000_000u128),
            slippage: "0.005".to_string(),
            ..Default::default()
        };
        let token_swap = SwapRequest {
            quote: QuoteRequest {
                from_token_amount: BigUint::from(100_000_000u64),
                ..Default::default()
            },
            amount_out_min: BigUint::from(99_213_808_000_000_000_000u128),
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

    #[test]
    fn test_is_valid_to_token() {
        assert!(is_valid_to_token("USDT(BSC)|other|0.005|bridgers|0", "USDT(BSC)", "0.005"));
        assert!(!is_valid_to_token("USDT(BSC)|ht6zut|0.01|bridgers|0", "USDT(BSC)", "0.005"));
        assert!(!is_valid_to_token("BNB(BSC)|ht6zut|0.005|bridgers|0", "USDT(BSC)", "0.005"));
        assert!(!is_valid_to_token("USDT(BSC)", "USDT(BSC)", "0.005"));
    }
}
