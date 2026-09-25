use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::swap::{SwapData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId, Currency};
use swapper::SwapperError;

use super::rules;
use super::session::{GemSwapProviderRow, provider_row};
use crate::duration_formatter::estimated_duration_parts;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::models::swap::GemSwapValue;
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
    pub to_asset: Asset,
    pub min_receive_value: GemBigUint,
    pub rate: Option<GemSwapRate>,
    pub price_impact: Option<SwapPriceImpact>,
    pub price_impact_row: Option<GemSwapPriceImpactRow>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPriceImpactRow {
    pub value: GemFormattedNumber,
    pub shows_in_summary: bool,
    pub warning: Option<GemLocalizedText>,
}

impl GemSwapQuoteSummary {
    /// Every detail row but the provider and the rate, which each app renders richly.
    pub fn detail_rows(&self, has_selected_slippage: bool) -> Vec<GemListRow> {
        let receive_asset = &self.to_asset;
        let price_impact = self.price_impact.filter(|impact| impact.shows_in_summary);
        [
            self.quote.eta_in_seconds.and_then(|seconds| estimated_duration_parts(seconds as i64)).map(|parts| GemListRow::Duration {
                title: GemListRowTitle::EstimatedTime,
                parts,
                info: None,
                estimate: false,
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
                amount: GemFormattedNumber::asset_amount(&BigInt::from(self.min_receive_value.clone()), receive_asset, GemValueStyle::Auto),
                info: None,
            }),
            Some(GemListRow::Label {
                title: GemListRowTitle::Slippage,
                text: match has_selected_slippage {
                    true => GemLocalizedText::Number {
                        number: GemFormattedNumber::percentage(rules::slippage_percent(self.quote.slippage_bps), GemPercentageStyle::Unsigned),
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
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapDetails {
    pub summary: GemSwapQuoteSummary,
    pub provider: GemSwapProviderRow,
    pub rows: Vec<GemListRow>,
}

#[uniffi::export]
pub fn swap_quote_details(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>, currency: Currency) -> GemSwapDetails {
    quote_details(quote, from_asset, to_asset, from_price, to_price, &currency, true)
}

pub fn quote_details(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>, currency: &Currency, has_selected_slippage: bool) -> GemSwapDetails {
    let provider = provider_row(quote.provider_data.provider, quote.provider_data.protocol_name.clone(), &quote.to_value, &to_asset, to_price, currency, false);
    let summary = swap_quote_summary(quote, from_asset, to_asset, from_price, to_price);
    GemSwapDetails {
        rows: summary.detail_rows(has_selected_slippage),
        provider,
        summary,
    }
}

pub fn swap_quote_summary(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>) -> GemSwapQuoteSummary {
    let pay = GemSwapValue::new(quote.from_value.clone(), from_asset.decimals as u32, from_price);
    let receive = GemSwapValue::new(quote.to_value.clone(), to_asset.decimals as u32, to_price);
    let price_impact = pay.price_impact(&receive);
    GemSwapQuoteSummary {
        min_receive_value: rules::min_receive_value(&quote.to_value, quote.slippage_bps),
        rate: rules::swap_rate(&from_asset, &quote.from_value, &to_asset, &quote.to_value),
        price_impact_row: price_impact.map(|impact| rules::price_impact_row(impact, from_asset.symbol.clone())),
        price_impact,
        to_asset,
        quote,
    }
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
    use super::{GemAssetRate, GemSwapQuoteSummary, swap_quote_summary};
    use primitives::{Asset, Chain, SwapProvider, SwapQuote};

    #[test]
    fn test_the_estimated_time_row_shows_only_a_time_it_can_read() {
        use crate::models::list::{GemListRow, GemListRowTitle};

        let estimated_time = |eta_in_seconds| {
            let quote = SwapQuote {
                eta_in_seconds,
                ..SwapQuote::mock_with_provider(SwapProvider::Okx)
            };
            swap_quote_summary(quote, Asset::from_chain(Chain::Solana), Asset::from_chain(Chain::Solana), None, None)
                .detail_rows(false)
                .into_iter()
                .find_map(|row| match row {
                    GemListRow::Duration {
                        title: GemListRowTitle::EstimatedTime,
                        parts,
                        ..
                    } => Some(parts),
                    _ => None,
                })
        };

        assert!(estimated_time(Some(90)).is_some_and(|parts| !parts.is_empty()));
        assert_eq!(estimated_time(Some(0)), None, "a zero estimate is no estimate, not an empty row");
        assert_eq!(estimated_time(None), None);
    }

    #[test]
    fn test_the_slippage_row_reads_auto_until_the_user_picks_one() {
        use crate::models::list::{GemListRow, GemListRowTitle};
        use crate::services::localization::GemLocalizedText;

        let quote = SwapQuote::mock_with_provider(SwapProvider::UniswapV3);
        let summary = swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana), None, None);
        let slippage = |has_selected| {
            summary
                .detail_rows(has_selected)
                .into_iter()
                .find_map(|row| match row {
                    GemListRow::Label { title: GemListRowTitle::Slippage, text, .. } => Some(text),
                    _ => None,
                })
                .unwrap()
        };

        assert_eq!(slippage(false), GemLocalizedText::SlippageAuto);
        assert!(
            matches!(slippage(true), GemLocalizedText::Number { number } if number.value == super::rules::slippage_percent(summary.quote.slippage_bps)),
            "a chosen slippage reads as the percent the quote was priced with"
        );
    }

    #[test]
    fn test_an_impact_the_screen_hides_leaves_no_row() {
        use crate::models::list::{GemListRow, GemListRowTitle};
        use primitives::swap::{SwapPriceImpact, SwapPriceImpactType};

        let quote = SwapQuote::mock_with_provider(SwapProvider::UniswapV3);
        let summary = swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana), None, None);
        let impact = |shows_in_summary| SwapPriceImpact {
            percentage: -5.0,
            impact_type: SwapPriceImpactType::High,
            is_high: true,
            shows_in_summary,
        };
        let has_impact_row = |impact| {
            GemSwapQuoteSummary {
                price_impact: Some(impact),
                ..summary.clone()
            }
            .detail_rows(false)
            .iter()
            .any(|row| matches!(row, GemListRow::Label { title: GemListRowTitle::PriceImpact, .. }))
        };

        assert!(has_impact_row(impact(true)));
        assert!(!has_impact_row(impact(false)), "the screen decides whether an impact is worth a row, once");
    }

    #[test]
    fn test_the_summary_prices_the_impact_from_both_sides() {
        let quote = SwapQuote {
            from_value: 10u64.pow(18).into(),
            to_value: 90u64.pow(9).into(),
            ..SwapQuote::mock_with_provider(SwapProvider::UniswapV3)
        };
        let eth = Asset::from_chain(Chain::Ethereum);
        let sol = Asset::from_chain(Chain::Solana);

        let priced = swap_quote_summary(quote.clone(), eth.clone(), sol.clone(), Some(100.0), Some(1.0));
        assert!(priced.price_impact.is_some());
        assert!(priced.price_impact_row.is_some());
        assert!(swap_quote_summary(quote, eth, sol, None, Some(1.0)).price_impact.is_none(), "an unpriced side has no impact");
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
