use crate::precision::GemPrecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPercentageStyle {
    Signed,
    Unsigned,
    UnsignedCompact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemPercentageFormat {
    pub precision: GemPrecision,
    pub shows_sign: bool,
}

#[uniffi::export]
impl GemPercentageStyle {
    pub fn format(&self) -> GemPercentageFormat {
        match self {
            Self::Signed => GemPercentageFormat {
                precision: GemPrecision::Fraction { min: 2, max: 2 },
                shows_sign: true,
            },
            Self::Unsigned => GemPercentageFormat {
                precision: GemPrecision::Fraction { min: 2, max: 2 },
                shows_sign: false,
            },
            Self::UnsignedCompact => GemPercentageFormat {
                precision: GemPrecision::Fraction { min: 0, max: 2 },
                shows_sign: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_the_signed_style_keeps_a_sign_and_only_the_compact_one_drops_trailing_zeros() {
        assert!(GemPercentageStyle::Signed.format().shows_sign);
        assert!(!GemPercentageStyle::Unsigned.format().shows_sign);
        assert_eq!(GemPercentageStyle::Unsigned.format().precision, GemPrecision::Fraction { min: 2, max: 2 });
        assert_eq!(GemPercentageStyle::UnsignedCompact.format().precision, GemPrecision::Fraction { min: 0, max: 2 });
    }
}
