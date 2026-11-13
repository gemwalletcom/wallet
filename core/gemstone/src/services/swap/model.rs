use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::GemTransferData;
use primitives::swap::{SlippageMode, SwapQuote};
use primitives::{Asset, AssetId, Currency};
use swapper::fees::subtract_bps;
use swapper::{Quote, SwapperError};

use super::rules;
use super::session::provider_row;
use crate::duration_formatter::estimated_duration_parts;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::GemProviderRow;
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::models::swap::GemSwapValue;
use crate::percentage::GemPercentageStyle;
use crate::services::localization::GemLocalizedText;
use num_bigint::BigInt;
use number_formatter::ValueStyle;

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
    pub price_impact_row: Option<GemSwapPriceImpactRow>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPriceImpactRow {
    pub value: GemFormattedNumber,
    pub shows_in_summary: bool,
    pub warning: Option<GemLocalizedText>,
}

impl GemSwapQuoteSummary {
    pub fn detail_rows(&self) -> Vec<GemListRow> {
        let receive_asset = &self.to_asset;
        [
            self.rate.clone().map(|rate| GemListRow::Rate {
                title: GemListRowTitle::Rate,
                rate: rate.direct,
                inverse: Some(rate.inverse),
            }),
            self.quote.eta_in_seconds.and_then(|seconds| estimated_duration_parts(seconds as i64)).map(|parts| GemListRow::Duration {
                title: GemListRowTitle::EstimatedTime,
                parts,
                info: None,
                estimate: false,
            }),
            self.price_impact_row.as_ref().filter(|row| row.shows_in_summary).map(|row| GemListRow::Label {
                title: GemListRowTitle::PriceImpact,
                text: GemLocalizedText::Number { number: row.value.clone() },
                tone: row.value.tone,
                info: Some(GemInfoTopic::PriceImpact),
                progress: false,
            }),
            Some(GemListRow::Amount {
                title: GemListRowTitle::MinimumReceive,
                amount: GemFormattedNumber::asset_amount(&BigInt::from(self.min_receive_value.clone()), receive_asset, ValueStyle::Auto),
                info: None,
            }),
            Some(GemListRow::Label {
                title: GemListRowTitle::Slippage,
                text: match self.quote.slippage_mode {
                    SlippageMode::Exact => GemLocalizedText::Number {
                        number: GemFormattedNumber::percentage(rules::slippage_percent(self.quote.slippage_bps), GemPercentageStyle::Unsigned),
                    },
                    SlippageMode::Auto => GemLocalizedText::SlippageAuto,
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
    pub provider: GemProviderRow,
    pub rows: Vec<GemListRow>,
}

#[uniffi::export]
pub fn swap_quote_details(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>, currency: Currency) -> GemSwapDetails {
    quote_details(quote, from_asset, to_asset, from_price, to_price, &currency)
}

pub fn quote_details(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>, currency: &Currency) -> GemSwapDetails {
    let provider = provider_row(quote.provider_data.provider, quote.provider_data.protocol_name.clone(), &quote.to_value, &to_asset, to_price, currency, false);
    let summary = swap_quote_summary(quote, from_asset, to_asset, from_price, to_price);
    GemSwapDetails {
        rows: summary.detail_rows(),
        provider,
        summary,
    }
}

pub fn swap_quote_summary(quote: SwapQuote, from_asset: Asset, to_asset: Asset, from_price: Option<f64>, to_price: Option<f64>) -> GemSwapQuoteSummary {
    let pay = GemSwapValue::new(quote.from_value.clone(), from_asset.decimals, from_price);
    let receive = GemSwapValue::new(quote.to_value.clone(), to_asset.decimals, to_price);
    GemSwapQuoteSummary {
        min_receive_value: subtract_bps(&quote.to_value, quote.slippage_bps),
        rate: rules::swap_rate(&from_asset, &quote.from_value, &to_asset, &quote.to_value),
        price_impact_row: pay.price_impact(&receive).map(|impact| rules::price_impact_row(impact, from_asset.symbol.clone())),
        to_asset,
        quote,
    }
}

#[derive(Debug, Clone)]
pub struct GemSwapRequote {
    pub quote: Quote,
    pub transfer: GemTransferData,
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
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapButtonAction {
    Swap,
    RetryQuote,
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
                .detail_rows()
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
        use primitives::swap::SlippageMode;

        let slippage = |slippage_mode| {
            let quote = SwapQuote {
                slippage_mode,
                ..SwapQuote::mock_with_provider(SwapProvider::UniswapV3)
            };
            swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana), None, None)
                .detail_rows()
                .into_iter()
                .find_map(|row| match row {
                    GemListRow::Label { title: GemListRowTitle::Slippage, text, .. } => Some(text),
                    _ => None,
                })
                .unwrap()
        };

        assert_eq!(slippage(SlippageMode::Auto), GemLocalizedText::SlippageAuto);
        assert!(
            matches!(slippage(SlippageMode::Exact), GemLocalizedText::Number { number } if number.value == super::rules::slippage_percent(SwapQuote::mock().slippage_bps)),
            "a chosen slippage reads as the percent the quote was priced with"
        );
    }

    #[test]
    fn test_the_details_sheet_colors_the_price_impact_as_the_details_row_does() {
        use crate::formatted_number::GemValueTone;
        use crate::models::list::{GemListRow, GemListRowTitle};

        let summary = |to_value: u64| {
            let quote = SwapQuote {
                from_value: 10u64.pow(18).into(),
                to_value: to_value.into(),
                ..SwapQuote::mock_with_provider(SwapProvider::UniswapV3)
            };
            swap_quote_summary(quote, Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Solana), Some(100.0), Some(1.0))
        };
        let sheet_tone = |summary: &GemSwapQuoteSummary| {
            summary.detail_rows().into_iter().find_map(|row| match row {
                GemListRow::Label {
                    title: GemListRowTitle::PriceImpact,
                    tone,
                    ..
                } => Some(tone),
                _ => None,
            })
        };
        let high = summary(90_510_000_000);

        assert_eq!(high.price_impact_row.as_ref().map(|row| row.value.tone), Some(GemValueTone::Negative));
        assert_eq!(sheet_tone(&high), Some(GemValueTone::Negative), "a 9.49% loss is red in the sheet as in Details, though it stays under the 10% warning");
        assert_eq!(sheet_tone(&summary(97_000_000_000)), Some(GemValueTone::Warning));
        assert_eq!(sheet_tone(&summary(99_500_000_000)), None, "a loss under 1% leaves no row");
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

        assert!(swap_quote_summary(quote.clone(), eth.clone(), sol.clone(), Some(100.0), Some(1.0)).price_impact_row.is_some());
        assert!(swap_quote_summary(quote, eth, sol, None, Some(1.0)).price_impact_row.is_none(), "an unpriced side has no impact");
    }

    #[test]
    fn test_the_details_sheet_opens_with_the_rate_both_ways() {
        use crate::models::list::{GemListRow, GemListRowTitle};

        let summary = swap_quote_summary(
            SwapQuote::mock_with_provider(SwapProvider::UniswapV3),
            Asset::from_chain(Chain::Ethereum),
            Asset::from_chain(Chain::Solana),
            None,
            None,
        );
        let rate = summary.rate.clone().unwrap();

        assert_eq!(
            summary.detail_rows().first(),
            Some(&GemListRow::Rate {
                title: GemListRowTitle::Rate,
                rate: rate.direct,
                inverse: Some(rate.inverse),
            })
        );
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
