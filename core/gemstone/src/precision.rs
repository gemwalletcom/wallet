use number_formatter::{Precision, ValueStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPrecision {
    Fraction { min: u32, max: u32 },
    Significant { max: u32 },
}

impl From<Precision> for GemPrecision {
    fn from(precision: Precision) -> Self {
        match precision {
            Precision::Fraction { min, max } => Self::Fraction { min, max },
            Precision::Significant { max } => Self::Significant { max },
        }
    }
}

#[uniffi::export]
pub fn abbreviation_threshold() -> f64 {
    number_formatter::ABBREVIATION_THRESHOLD
}

#[uniffi::export]
pub fn adaptive_precision(magnitude: f64) -> GemPrecision {
    number_formatter::precision::adaptive(magnitude).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemValueStyle {
    Full,
    Short,
    Auto,
}

impl From<GemValueStyle> for ValueStyle {
    fn from(style: GemValueStyle) -> Self {
        match style {
            GemValueStyle::Full => Self::Full,
            GemValueStyle::Short => Self::Short,
            GemValueStyle::Auto => Self::Auto,
        }
    }
}

#[uniffi::export]
impl GemValueStyle {
    pub fn precision(&self, magnitude: f64) -> GemPrecision {
        number_formatter::precision::value((*self).into(), magnitude).into()
    }

    pub fn abbreviates(&self, magnitude: f64) -> bool {
        number_formatter::precision::abbreviates_value((*self).into(), magnitude)
    }

    pub fn is_dust(&self, magnitude: f64) -> bool {
        number_formatter::precision::is_value_dust((*self).into(), magnitude)
    }
}

#[uniffi::export]
pub fn dust_threshold() -> f64 {
    number_formatter::VALUE_DUST_THRESHOLD
}

#[uniffi::export]
pub fn dust_threshold_places() -> u32 {
    number_formatter::VALUE_DUST_PLACES
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCurrencyStyle {
    Currency,
    Fiat,
    Abbreviated,
}

#[uniffi::export]
impl GemCurrencyStyle {
    pub fn precision(&self, magnitude: f64) -> GemPrecision {
        match self {
            Self::Fiat => number_formatter::Precision::TWO_PLACES.into(),
            Self::Currency | Self::Abbreviated => number_formatter::precision::adaptive(magnitude).into(),
        }
    }

    pub fn abbreviates(&self, magnitude: f64) -> bool {
        match self {
            Self::Abbreviated => magnitude.abs() >= number_formatter::ABBREVIATION_THRESHOLD,
            Self::Currency | Self::Fiat => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_value_style_answers_the_ladder_the_abbreviation_and_the_dust() {
        assert_eq!(GemValueStyle::Full.precision(0.00001), GemPrecision::Fraction { min: 0, max: 32 });
        assert_eq!(GemValueStyle::Short.precision(0.09), GemPrecision::Fraction { min: 0, max: 4 });
        assert_eq!(GemValueStyle::Auto.precision(0.99), GemPrecision::Significant { max: 4 });

        assert!(GemValueStyle::Short.abbreviates(100_000.0));
        assert!(!GemValueStyle::Auto.abbreviates(100_000.0));

        assert!(GemValueStyle::Short.is_dust(0.00009));
        assert!(!GemValueStyle::Short.is_dust(0.0));
        assert_eq!(dust_threshold(), 0.0001);
        assert_eq!(dust_threshold_places(), 4);
    }

    #[test]
    fn test_fiat_always_reads_in_two_places_and_only_abbreviated_compacts() {
        assert_eq!(GemCurrencyStyle::Fiat.precision(0.0001), GemPrecision::Fraction { min: 2, max: 2 });
        assert_eq!(GemCurrencyStyle::Currency.precision(0.5), GemPrecision::Significant { max: 4 });

        assert!(GemCurrencyStyle::Abbreviated.abbreviates(100_000.0));
        assert!(!GemCurrencyStyle::Abbreviated.abbreviates(99_999.0));
        assert!(!GemCurrencyStyle::Currency.abbreviates(1_000_000.0));
    }
}
