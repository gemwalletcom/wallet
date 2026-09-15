use crate::percentage::GemPercentageStyle;
use crate::precision::{GemCurrencyStyle, GemPrecision, GemValueStyle};
use primitives::Currency;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberUnit {
    Currency { code: String },
    Symbol { symbol: String },
    Percent,
    Plain,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberNotation {
    Plain,
    Signed,
    Parenthesised,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNumberDisplay {
    Number { precision: GemPrecision },
    Abbreviated,
    BelowThreshold { threshold: f64, places: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFormattedNumber {
    pub value: f64,
    pub unit: GemNumberUnit,
    pub display: GemNumberDisplay,
    pub notation: GemNumberNotation,
}

impl GemFormattedNumber {
    pub fn currency(value: f64, currency: Currency, style: GemCurrencyStyle) -> Self {
        Self::currency_code(value, currency.as_ref().to_string(), style)
    }

    pub fn currency_code(value: f64, code: String, style: GemCurrencyStyle) -> Self {
        Self {
            value,
            notation: GemNumberNotation::Plain,
            unit: GemNumberUnit::Currency { code },
            display: match style.abbreviates(value) {
                true => GemNumberDisplay::Abbreviated,
                false => GemNumberDisplay::Number {
                    precision: style.precision(value),
                },
            },
        }
    }

    pub fn usd(value: f64) -> Self {
        Self::currency(value, Currency::USD, GemCurrencyStyle::Currency)
    }

    pub fn signed_usd(value: f64) -> Self {
        Self::signed_currency(value, Currency::USD, GemCurrencyStyle::Currency)
    }

    pub fn usd_abbreviated(value: f64) -> Self {
        Self::currency(value, Currency::USD, GemCurrencyStyle::Abbreviated)
    }

    pub fn in_parentheses(self) -> Self {
        Self {
            notation: GemNumberNotation::Parenthesised,
            ..self
        }
    }

    pub fn signed_currency(value: f64, currency: Currency, style: GemCurrencyStyle) -> Self {
        Self {
            notation: GemNumberNotation::Signed,
            ..Self::currency(value, currency, style)
        }
    }

    pub fn adaptive(value: f64, symbol: Option<String>) -> Self {
        Self {
            value,
            notation: GemNumberNotation::Plain,
            unit: unit(symbol),
            display: GemNumberDisplay::Number {
                precision: crate::precision::adaptive_precision(value),
            },
        }
    }

    pub fn percentage(value: f64, style: GemPercentageStyle) -> Self {
        let format = style.format();
        Self {
            value,
            notation: notation(format.shows_sign),
            unit: GemNumberUnit::Percent,
            display: GemNumberDisplay::Number { precision: format.precision },
        }
    }

    pub fn amount(value: f64, symbol: Option<String>, style: GemValueStyle) -> Self {
        Self {
            value,
            notation: GemNumberNotation::Plain,
            unit: unit(symbol),
            display: value_display(value, style),
        }
    }
}

fn notation(shows_sign: bool) -> GemNumberNotation {
    match shows_sign {
        true => GemNumberNotation::Signed,
        false => GemNumberNotation::Plain,
    }
}

fn unit(symbol: Option<String>) -> GemNumberUnit {
    match symbol {
        Some(symbol) => GemNumberUnit::Symbol { symbol },
        None => GemNumberUnit::Plain,
    }
}

#[uniffi::export]
pub fn formatted_adaptive(value: f64, symbol: Option<String>) -> GemFormattedNumber {
    GemFormattedNumber::adaptive(value, symbol)
}

#[uniffi::export]
pub fn formatted_currency(value: f64, code: String, style: GemCurrencyStyle) -> GemFormattedNumber {
    GemFormattedNumber::currency_code(value, code, style)
}

#[uniffi::export]
pub fn formatted_amount(value: f64, symbol: Option<String>, style: GemValueStyle) -> GemFormattedNumber {
    GemFormattedNumber::amount(value, symbol, style)
}

fn value_display(value: f64, style: GemValueStyle) -> GemNumberDisplay {
    if style.abbreviates(value) {
        return GemNumberDisplay::Abbreviated;
    }
    if style.is_dust(value) {
        return GemNumberDisplay::BelowThreshold {
            threshold: number_formatter::VALUE_DUST_THRESHOLD,
            places: number_formatter::VALUE_DUST_PLACES,
        };
    }
    GemNumberDisplay::Number {
        precision: style.precision(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_currency_number_carries_its_code_and_its_precision() {
        let fiat = GemFormattedNumber::currency(12.3456, Currency::USD, GemCurrencyStyle::Fiat);
        assert_eq!(fiat.value, 12.3456);
        assert_eq!(fiat.unit, GemNumberUnit::Currency { code: "USD".to_string() });
        assert_eq!(
            fiat.display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Fraction { min: 2, max: 2 }
            }
        );
    }

    #[test]
    fn test_only_an_abbreviating_style_reads_as_abbreviated() {
        assert_eq!(
            GemFormattedNumber::currency(1_000_000.0, Currency::USD, GemCurrencyStyle::Abbreviated).display,
            GemNumberDisplay::Abbreviated
        );
        assert_eq!(
            GemFormattedNumber::currency(1_000_000.0, Currency::USD, GemCurrencyStyle::Currency).display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Fraction { min: 2, max: 2 }
            }
        );
    }

    #[test]
    fn test_a_percentage_carries_its_sign_rule_with_it() {
        let signed = GemFormattedNumber::percentage(-2.0, GemPercentageStyle::Signed);
        assert_eq!(signed.value, -2.0);
        assert_eq!(signed.unit, GemNumberUnit::Percent);
        assert_eq!(signed.notation, GemNumberNotation::Signed);
        assert_eq!(
            signed.display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Fraction { min: 2, max: 2 }
            }
        );

        assert_eq!(GemFormattedNumber::percentage(5.0, GemPercentageStyle::Unsigned).notation, GemNumberNotation::Plain);
    }

    #[test]
    fn test_an_adaptive_number_keeps_two_places_from_one_up_and_four_digits_below() {
        assert_eq!(
            GemFormattedNumber::adaptive(100.0, Some("USDT".to_string())).display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Fraction { min: 2, max: 2 }
            }
        );
        assert_eq!(
            GemFormattedNumber::adaptive(0.000838216, None).display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Significant { max: 4 }
            }
        );
    }

    #[test]
    fn test_an_amount_carries_its_symbol_and_reads_dust_below_the_threshold() {
        let dust = GemFormattedNumber::amount(0.00001, Some("BTC".to_string()), GemValueStyle::Short);
        assert_eq!(dust.unit, GemNumberUnit::Symbol { symbol: "BTC".to_string() });
        assert_eq!(dust.display, GemNumberDisplay::BelowThreshold { threshold: 0.0001, places: 4 });

        let plain = GemFormattedNumber::amount(0.5, None, GemValueStyle::Auto);
        assert_eq!(plain.unit, GemNumberUnit::Plain);
        assert_eq!(
            plain.display,
            GemNumberDisplay::Number {
                precision: GemPrecision::Significant { max: 4 }
            }
        );
    }
}
