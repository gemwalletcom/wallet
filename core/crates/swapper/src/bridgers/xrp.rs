use gem_xrp::models::Amount;
use num_bigint::BigUint;
use primitives::AssetId;

use super::{
    model::{DepositMemo, SwapRequest, XrpTransaction},
    transaction::get_to_token_channel,
};
use crate::SwapperError;

pub(super) fn get_transaction_memo(transaction: &XrpTransaction, router: &str, from_asset: &AssetId, value: &BigUint, to_code: &str, swap: &SwapRequest) -> Result<String, SwapperError> {
    let is_amount = match (&transaction.amount, &from_asset.token_id) {
        (Amount::Str(amount), None) => amount == value,
        (Amount::Amount(amount), Some(token_id)) => amount.issuer.as_ref() == Some(token_id) && &amount.value == value,
        _ => false,
    };
    if transaction.destination != router || !is_amount {
        return Err(SwapperError::InvalidRoute);
    }
    let [memo] = transaction.memos.as_slice() else {
        return Err(SwapperError::InvalidRoute);
    };
    let data = hex::decode(memo.memo.data.as_deref().ok_or(SwapperError::InvalidRoute)?).map_err(|_| SwapperError::InvalidRoute)?;
    let memo: DepositMemo = serde_json::from_slice(&data).map_err(|_| SwapperError::InvalidRoute)?;
    let channel = get_to_token_channel(&memo.to_token, to_code, &swap.slippage)?;
    let expected = DepositMemo {
        from_amount: Some(swap.quote.from_token_amount.to_string()),
        ..DepositMemo::new(swap, to_code, channel)
    };
    if memo != expected {
        return Err(SwapperError::InvalidRoute);
    }
    serde_json::to_string(&memo).map_err(SwapperError::compute_quote_error)
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, asset_constants::XRP_RLUSD_ASSET_ID};

    use super::*;
    use crate::bridgers::{
        asset::get_token_address,
        model::{BridgersResponse, QuoteRequest, SwapData, SwapTransaction},
        testkit::{TEST_EVM_WALLET, TEST_XRP_WALLET},
    };

    const ROUTER: &str = "rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn";

    fn transaction(json: &str) -> XrpTransaction {
        let response: BridgersResponse = serde_json::from_str(json).unwrap();
        let SwapTransaction::Xrp(transaction) = serde_json::from_value::<SwapData>(response.data).unwrap().tx_data else {
            panic!()
        };
        transaction
    }

    fn swap(from_token_address: &str, from_token_amount: u64) -> SwapRequest {
        SwapRequest {
            quote: QuoteRequest {
                from_token_address: from_token_address.to_string(),
                from_token_amount: BigUint::from(from_token_amount),
                ..Default::default()
            },
            from_address: TEST_XRP_WALLET.to_string(),
            to_address: TEST_EVM_WALLET.to_string(),
            amount_out_min: BigUint::from(1u32),
            slippage: "0.01".to_string(),
        }
    }

    #[test]
    fn test_get_transaction_memo() {
        let native = transaction(include_str!("testdata/swap_xrp_bsc_usdt.json"));
        let token = transaction(include_str!("testdata/swap_rlusd_bsc_usdt.json"));
        let xrp = AssetId::from_chain(Chain::Xrp);
        let native_swap = swap(&get_token_address(&xrp).unwrap(), 2_000_000);
        let token_swap = swap(&get_token_address(&XRP_RLUSD_ASSET_ID).unwrap(), 500_000_000_000_000_000);
        let native_value = BigUint::from(2_000_000u64);
        let token_value = BigUint::from(500_000_000_000_000u64);

        let memo: DepositMemo = serde_json::from_str(&get_transaction_memo(&native, ROUTER, &xrp, &native_value, "USDT(BSC)", &native_swap).unwrap()).unwrap();
        assert_eq!(memo.to_token, "USDT(BSC)|ik9r6b|0.01|bridgers|0");
        assert_eq!(memo.from_amount.as_deref(), Some("2000000"));

        let memo: DepositMemo = serde_json::from_str(&get_transaction_memo(&token, ROUTER, &XRP_RLUSD_ASSET_ID, &token_value, "USDT(BSC)", &token_swap).unwrap()).unwrap();
        assert_eq!(memo.from_amount.as_deref(), Some("500000000000000000"));

        assert_eq!(get_transaction_memo(&native, TEST_XRP_WALLET, &xrp, &native_value, "USDT(BSC)", &native_swap).unwrap_err(), SwapperError::InvalidRoute);
        assert_eq!(get_transaction_memo(&native, ROUTER, &xrp, &BigUint::from(1u32), "USDT(BSC)", &native_swap).unwrap_err(), SwapperError::InvalidRoute);
        assert_eq!(get_transaction_memo(&native, ROUTER, &XRP_RLUSD_ASSET_ID, &native_value, "USDT(BSC)", &native_swap).unwrap_err(), SwapperError::InvalidRoute);
        assert_eq!(get_transaction_memo(&native, ROUTER, &xrp, &native_value, "BNB(BSC)", &native_swap).unwrap_err(), SwapperError::InvalidRoute);
        assert_eq!(
            get_transaction_memo(
                &native,
                ROUTER,
                &xrp,
                &native_value,
                "USDT(BSC)",
                &SwapRequest {
                    amount_out_min: BigUint::from(2u32),
                    ..native_swap.clone()
                }
            )
            .unwrap_err(),
            SwapperError::InvalidRoute
        );
    }
}
