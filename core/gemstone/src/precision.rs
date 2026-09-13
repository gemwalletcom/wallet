use number_formatter::Precision;

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
pub fn adaptive_precision(magnitude: f64) -> GemPrecision {
    number_formatter::precision::adaptive(magnitude).into()
}
