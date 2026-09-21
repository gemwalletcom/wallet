use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::swap::{SwapData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId};
use swapper::{Quote, SwapperError};

use super::rules;
use crate::formatted_number::GemFormattedNumber;
use primitives::TransactionInputType;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetRate {
    pub base_symbol: String,
    pub value: GemFormattedNumber,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapDetailRow {
    Provider,
    Rate,
    EstimatedTime,
    PriceImpact,
    MinimumReceive,
    Slippage,
}

#[uniffi::export]
impl GemSwapQuoteSummary {
    pub fn slippage_percent(&self) -> f64 {
        rules::slippage_percent(self.quote.slippage_bps)
    }

    pub fn rows(&self, shows_price_impact: bool) -> Vec<GemSwapDetailRow> {
        [
            Some(GemSwapDetailRow::Provider),
            self.rate.is_some().then_some(GemSwapDetailRow::Rate),
            self.quote.eta_in_seconds.is_some().then_some(GemSwapDetailRow::EstimatedTime),
            shows_price_impact.then_some(GemSwapDetailRow::PriceImpact),
            Some(GemSwapDetailRow::MinimumReceive),
            Some(GemSwapDetailRow::Slippage),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
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

#[derive(Debug, Clone, PartialEq)]
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
    use super::{GemAssetRate, GemSwapDetailRow, swap_quote_summary};
    use primitives::{Asset, Chain, SwapProvider, SwapQuote};

    #[test]
    fn test_the_details_list_drops_the_rows_a_quote_cannot_fill() {
        let quote = SwapQuote::mock_with_provider(SwapProvider::UniswapV3);
        let summary = swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana));

        let rows = summary.rows(false);
        assert_eq!(rows.first(), Some(&GemSwapDetailRow::Provider));
        assert!(!rows.contains(&GemSwapDetailRow::PriceImpact), "the impact row shows only when the screen has prices for it");
        assert_eq!(rows.last(), Some(&GemSwapDetailRow::Slippage));
        assert!(summary.rows(true).contains(&GemSwapDetailRow::PriceImpact));
    }

    #[test]
    fn test_rate_text_names_the_base_and_keeps_the_formatted_value() {
        let rate = GemAssetRate {
            base_symbol: "BTC".to_string(),
            value: crate::formatted_number::GemFormattedNumber::adaptive(100.0, Some("USDT".to_string())),
        };
        assert_eq!(rate.text("100.00 USDT".to_string()), "1 BTC ≈ 100.00 USDT");
    }
}
