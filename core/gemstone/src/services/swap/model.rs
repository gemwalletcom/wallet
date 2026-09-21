use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::swap::{SwapData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId};
use swapper::{Quote, SwapperError};

use super::rules;
use crate::duration_formatter::DurationFormatter;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::percentage::GemPercentageStyle;
use crate::precision::GemValueStyle;
use crate::services::localization::GemLocalizedText;
use num_bigint::BigInt;
use primitives::TransactionInputType;
use primitives::swap::SwapPriceImpact;

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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPriceImpactRow {
    pub value: GemFormattedNumber,
    pub shows_in_summary: bool,
    pub warning: Option<GemLocalizedText>,
}

#[uniffi::export]
pub fn swap_price_impact_row(impact: SwapPriceImpact, pay_symbol: String) -> GemSwapPriceImpactRow {
    rules::price_impact_row(impact, pay_symbol)
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

    /// Every detail row but the provider and the rate, which each app renders richly.
    pub fn detail_rows(&self, receive_asset: Asset, price_impact: Option<SwapPriceImpact>, has_selected_slippage: bool) -> Vec<GemListRow> {
        let price_impact = price_impact.filter(|impact| impact.shows_in_summary);
        [
            self.quote.eta_in_seconds.map(|seconds| GemListRow::Duration {
                title: GemListRowTitle::EstimatedTime,
                parts: DurationFormatter::new().estimate_parts(seconds as i64),
                info: None,
            }),
            price_impact.map(|impact| GemListRow::Label {
                title: GemListRowTitle::PriceImpact,
                text: GemLocalizedText::Number {
                    number: GemFormattedNumber::percentage(impact.percentage, GemPercentageStyle::Signed),
                },
                tone: match impact.is_high {
                    true => GemValueTone::Negative,
                    false => GemValueTone::Plain,
                },
                info: Some(GemInfoTopic::PriceImpact),
                progress: false,
            }),
            Some(GemListRow::Amount {
                title: GemListRowTitle::MinimumReceive,
                amount: GemFormattedNumber::asset_amount(&BigInt::from(self.min_receive_value.clone()), &receive_asset, GemValueStyle::Auto),
                info: None,
            }),
            Some(GemListRow::Label {
                title: GemListRowTitle::Slippage,
                text: match has_selected_slippage {
                    true => GemLocalizedText::Number {
                        number: GemFormattedNumber::percentage(self.slippage_percent(), GemPercentageStyle::Unsigned),
                    },
                    false => GemLocalizedText::SlippageAuto,
                },
                tone: GemValueTone::Plain,
                info: Some(GemInfoTopic::Slippage),
                progress: false,
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
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
    fn test_the_slippage_row_reads_auto_until_the_user_picks_one() {
        use crate::models::list::{GemListRow, GemListRowTitle};
        use crate::services::localization::GemLocalizedText;

        let quote = SwapQuote::mock_with_provider(SwapProvider::UniswapV3);
        let summary = swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana));
        let slippage = |has_selected| {
            summary
                .detail_rows(Asset::from_chain(Chain::Solana), None, has_selected)
                .into_iter()
                .find_map(|row| match row {
                    GemListRow::Label { title: GemListRowTitle::Slippage, text, .. } => Some(text),
                    _ => None,
                })
                .unwrap()
        };

        assert_eq!(slippage(false), GemLocalizedText::SlippageAuto);
        assert!(
            matches!(slippage(true), GemLocalizedText::Number { number } if number.value == summary.slippage_percent()),
            "a chosen slippage reads as the percent the quote was priced with"
        );
    }

    #[test]
    fn test_an_impact_the_screen_hides_leaves_no_row() {
        use crate::models::list::{GemListRow, GemListRowTitle};
        use primitives::swap::{SwapPriceImpact, SwapPriceImpactType};

        let quote = SwapQuote::mock_with_provider(SwapProvider::UniswapV3);
        let summary = swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana));
        let impact = |shows_in_summary| SwapPriceImpact {
            percentage: -5.0,
            impact_type: SwapPriceImpactType::High,
            is_high: true,
            shows_in_summary,
        };
        let has_impact_row = |impact| {
            summary
                .detail_rows(Asset::from_chain(Chain::Solana), Some(impact), false)
                .iter()
                .any(|row| matches!(row, GemListRow::Label { title: GemListRowTitle::PriceImpact, .. }))
        };

        assert!(has_impact_row(impact(true)));
        assert!(!has_impact_row(impact(false)), "the screen decides whether an impact is worth a row, once");
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
