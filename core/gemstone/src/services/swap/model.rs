use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::swap::{SwapData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId};
use swapper::{Quote, SwapperError};

use super::rules;
use primitives::TransactionInputType;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetRate {
    pub base_symbol: String,
    pub quote_symbol: String,
    pub value: f64,
}

#[uniffi::export]
impl GemAssetRate {
    pub fn text(&self, formatted_value: String) -> String {
        format!("1 {} ≈ {}", self.base_symbol, formatted_value)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapRate {
    pub direct: GemAssetRate,
    pub inverse: GemAssetRate,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapQuoteSummary {
    pub quote: SwapQuote,
    pub min_receive_value: GemBigUint,
    pub rate: Option<GemSwapRate>,
}

#[uniffi::export]
pub fn swap_quote_summary(quote: SwapQuote, from_asset: Asset, to_asset: Asset) -> GemSwapQuoteSummary {
    GemSwapQuoteSummary {
        min_receive_value: rules::min_receive_value(&quote.to_value, quote.slippage_bps),
        rate: rules::swap_rate(&from_asset, &quote.from_value, &to_asset, &quote.to_value),
        quote,
    }
}

#[uniffi::export]
pub fn swapper_quote_summary(quote: Quote, from_asset: Asset, to_asset: Asset) -> GemSwapQuoteSummary {
    swap_quote_summary(rules::swap_quote(&quote), from_asset, to_asset)
}

#[uniffi::export]
pub fn swap_quote(quote: Quote) -> SwapQuote {
    rules::swap_quote(&quote)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapTransfer {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
    pub recipient: String,
    pub value: GemBigUint,
    pub use_max_amount: bool,
}

#[uniffi::export]
impl GemSwapTransfer {
    pub fn transfer_data(&self, from_asset: Asset, to_asset: Asset) -> GemTransferData {
        GemTransferData {
            input_type: TransactionInputType::Swap {
                from_asset,
                to_asset,
                swap_data: SwapData {
                    quote: self.quote.clone(),
                    data: self.data.clone(),
                },
            },
            recipient: GemRecipient {
                address: self.recipient.clone(),
                name: None,
                memo: self.data.memo.clone(),
                references: vec![],
            },
            value: self.value.clone().into(),
            use_max_amount: self.use_max_amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPair {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapSide {
    Pay,
    Receive,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPairSelection {
    pub pay_asset_id: Option<AssetId>,
    pub receive_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPairSuggestion {
    pub pay_asset_id: AssetId,
    pub receive_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapButtonInput {
    pub value: GemBigInt,
    pub available_balance: GemBigInt,
    pub quote_error: Option<SwapperError>,
    pub transfer_error: Option<SwapperError>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapButtonAction {
    Swap,
    RetryQuote,
    RetryTransfer,
    UseMinimumAmount { value: GemBigInt },
    InsufficientBalance,
}

#[cfg(test)]
mod tests {
    use super::GemAssetRate;

    #[test]
    fn test_rate_text_names_the_base_and_keeps_the_formatted_value() {
        let rate = GemAssetRate {
            base_symbol: "BTC".to_string(),
            quote_symbol: "USDT".to_string(),
            value: 100.0,
        };
        assert_eq!(rate.text("100.00 USDT".to_string()), "1 BTC ≈ 100.00 USDT");
    }
}
