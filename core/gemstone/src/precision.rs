use number_formatter::Precision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPrecision {
    Fraction { min: usize, max: usize },
    Significant { max: usize },
}

impl From<Precision> for GemPrecision {
    fn from(precision: Precision) -> Self {
        match precision {
            Precision::Fraction { min, max } => Self::Fraction { min, max },
            Precision::Significant { max } => Self::Significant { max },
        }
    }
}

pub fn adaptive_precision(magnitude: f64) -> GemPrecision {
    number_formatter::precision::adaptive(magnitude).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCurrencyStyle {
    Currency,
    Short,
    Fiat,
    Abbreviated,
}

impl GemCurrencyStyle {
    pub fn precision(&self, magnitude: f64) -> GemPrecision {
        match self {
            Self::Fiat => number_formatter::Precision::TWO_PLACES.into(),
            Self::Currency | Self::Short | Self::Abbreviated => number_formatter::precision::currency(magnitude).into(),
        }
    }

    pub fn abbreviates(&self, magnitude: f64) -> bool {
        match self {
            Self::Abbreviated => magnitude.abs() >= number_formatter::ABBREVIATION_THRESHOLD,
            Self::Currency | Self::Short | Self::Fiat => false,
        }
    }

    pub fn is_dust(&self, magnitude: f64) -> bool {
        match self {
            Self::Short => number_formatter::precision::is_dust(magnitude),
            Self::Currency | Self::Fiat | Self::Abbreviated => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_always_reads_in_two_places_and_only_abbreviated_compacts() {
        assert_eq!(GemCurrencyStyle::Fiat.precision(0.0001), GemPrecision::Fraction { min: 2, max: 2 });
        assert_eq!(GemCurrencyStyle::Currency.precision(0.5), GemPrecision::Fraction { min: 2, max: 4 });

        assert!(GemCurrencyStyle::Abbreviated.abbreviates(100_000.0));
        assert!(!GemCurrencyStyle::Abbreviated.abbreviates(99_999.0));
        assert!(!GemCurrencyStyle::Currency.abbreviates(1_000_000.0));
        assert!(!GemCurrencyStyle::Short.abbreviates(1_000_000.0));
    }

    #[test]
    fn test_only_the_short_currency_style_reads_as_dust() {
        assert_eq!(GemCurrencyStyle::Short.precision(0.5), GemPrecision::Fraction { min: 2, max: 4 });
        assert!(GemCurrencyStyle::Short.is_dust(0.00000783));
        assert!(!GemCurrencyStyle::Short.is_dust(0.0001));
        assert!(!GemCurrencyStyle::Short.is_dust(0.0));
        assert!(!GemCurrencyStyle::Currency.is_dust(0.00000783));
        assert!(!GemCurrencyStyle::Fiat.is_dust(0.00000783));
        assert!(!GemCurrencyStyle::Abbreviated.is_dust(0.00000783));
    }
}
