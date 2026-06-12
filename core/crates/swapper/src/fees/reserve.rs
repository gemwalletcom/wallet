use alloy_primitives::U256;
use primitives::Chain;
use std::{collections::HashMap, str::FromStr, sync::LazyLock};

use crate::{QuoteRequest, SwapperError};

pub static RESERVED_NATIVE_FEES: LazyLock<HashMap<Chain, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (Chain::Ethereum, "1000000000000000"),   // 0.001 ETH
        (Chain::Arbitrum, "300000000000000"),    // 0.0003 ARB ETH
        (Chain::Base, "300000000000000"),        // 0.0003 BASE ETH
        (Chain::Optimism, "500000000000000"),    // 0.0005 OP ETH
        (Chain::AvalancheC, "3000000000000000"), // 0.003 AVAX
        (Chain::SmartChain, "2000000000000000"), // 0.002 BNB
        (Chain::Polygon, "20000000000000000"),   // 0.02 MATIC
        (Chain::Gnosis, "5000000000000000"),     // 0.005 XDAI
        (Chain::Berachain, "5000000000000000"),  // 0.005 BERA
        (Chain::Sui, "50000000"),                // 0.05 SUI
        (Chain::Solana, "5000000"),              // 0.005 SOL: base + priority fees + wSOL ATA rent
        (Chain::Ton, "20000000"),                // 0.02 TON
        (Chain::Aptos, "20000000"),              // 0.2 APT
        (Chain::Monad, "5000000000000000"),      // 0.005 MON
        (Chain::XLayer, "5000000000000000"),     // 0.005 OKB
        (Chain::Plasma, "5000000000000000"),     // 0.005 XPL
    ])
});

pub fn reserved_transaction_fees(chain: Chain) -> Option<&'static str> {
    RESERVED_NATIVE_FEES.get(&chain).copied()
}

fn quote_value_after_reserve(request: &QuoteRequest, reserved: &str) -> Result<String, SwapperError> {
    if !request.options.use_max_amount || !request.from_asset.asset_id().is_native() {
        return Ok(request.value.clone());
    }
    let reserved_fee = U256::from_str(reserved).map_err(|_| SwapperError::ComputeQuoteError(format!("invalid reserved fee: {reserved}")))?;
    let amount = U256::from_str(&request.value).map_err(|_| SwapperError::ComputeQuoteError(format!("invalid amount: {}", request.value)))?;
    if amount <= reserved_fee {
        return Err(SwapperError::InputAmountError {
            min_amount: Some(reserved_fee.to_string()),
        });
    }
    Ok((amount - reserved_fee).to_string())
}

pub fn max_quote_value_with_fee_reserve(request: &QuoteRequest) -> Result<String, SwapperError> {
    let Some(reserved) = reserved_transaction_fees(request.from_asset.chain()) else {
        return Ok(request.value.clone());
    };
    quote_value_after_reserve(request, reserved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset};
    use primitives::{AssetId, asset_constants::SOLANA_USDC_TOKEN_ID};

    #[test]
    fn solana_reserve_covers_rent_and_priority_fees() {
        let reserve: u64 = reserved_transaction_fees(Chain::Solana).unwrap().parse().unwrap();
        assert!(reserve >= 2_500_000, "Solana reserve {reserve} too small to cover wSOL ATA rent plus priority fees");
    }

    #[test]
    fn max_quote_value_applies_reserve_for_native_only() {
        let mut request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            wallet_address: "address".to_string(),
            destination_address: "address".to_string(),
            value: "105814789".to_string(),
            options: Options {
                use_max_amount: true,
                ..Default::default()
            },
        };

        assert_eq!(max_quote_value_with_fee_reserve(&request).unwrap(), "100814789");

        request.from_asset = SwapperQuoteAsset::from(AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID));
        assert_eq!(max_quote_value_with_fee_reserve(&request).unwrap(), "105814789");

        request.from_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana));
        request.options.use_max_amount = false;
        assert_eq!(max_quote_value_with_fee_reserve(&request).unwrap(), "105814789");
    }
}
