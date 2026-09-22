use crate::value_formatter::ValueStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Fraction { min: u32, max: u32 },
    Significant { max: u32 },
}

impl Precision {
    pub const TWO_PLACES: Self = Self::Fraction { min: 2, max: 2 };
    pub const FOUR_SIGNIFICANT: Self = Self::Significant { max: 4 };
    pub const UP_TO_TWO_PLACES: Self = Self::Fraction { min: 0, max: 2 };
    pub const UP_TO_FOUR_PLACES: Self = Self::Fraction { min: 0, max: 4 };
    pub const FULL: Self = Self::Fraction { min: 0, max: 32 };
}

pub const ABBREVIATION_THRESHOLD: f64 = 100_000.0;
pub const SMALL_AMOUNT_THRESHOLD: f64 = 0.1;
pub const VALUE_DUST_THRESHOLD: f64 = 1e-4;
pub const VALUE_DUST_PLACES: u32 = 4;

pub fn value(style: ValueStyle, magnitude: f64) -> Precision {
    let magnitude = magnitude.abs();
    match style {
        ValueStyle::Full => Precision::FULL,
        ValueStyle::Short if magnitude >= SMALL_AMOUNT_THRESHOLD => Precision::UP_TO_TWO_PLACES,
        ValueStyle::Short => Precision::UP_TO_FOUR_PLACES,
        ValueStyle::Auto if magnitude >= 1.0 => Precision::UP_TO_TWO_PLACES,
        ValueStyle::Auto => Precision::FOUR_SIGNIFICANT,
    }
}

pub fn abbreviates_value(style: ValueStyle, magnitude: f64) -> bool {
    matches!(style, ValueStyle::Short) && magnitude.abs() >= ABBREVIATION_THRESHOLD
}

pub fn is_value_dust(style: ValueStyle, magnitude: f64) -> bool {
    matches!(style, ValueStyle::Short) && is_dust(magnitude)
}

pub fn is_dust(magnitude: f64) -> bool {
    magnitude != 0.0 && magnitude.abs() < VALUE_DUST_THRESHOLD
}

const SMALL_VALUE_THRESHOLD: f64 = 0.99;
const DUST_THRESHOLD: f64 = 1e-10;

pub fn adaptive(magnitude: f64) -> Precision {
    match (DUST_THRESHOLD..SMALL_VALUE_THRESHOLD).contains(&magnitude.abs()) {
        true => Precision::FOUR_SIGNIFICANT,
        false => Precision::TWO_PLACES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_value_between_dust_and_one_reads_in_significant_digits() {
        assert_eq!(adaptive(0.000838216), Precision::FOUR_SIGNIFICANT);
        assert_eq!(adaptive(0.98), Precision::FOUR_SIGNIFICANT);
        assert_eq!(adaptive(-0.5), Precision::FOUR_SIGNIFICANT);
    }

    #[test]
    fn test_the_value_ladder_matches_on_both_apps() {
        assert_eq!(value(ValueStyle::Full, 0.00001), Precision::FULL);
        assert_eq!(value(ValueStyle::Short, 0.1), Precision::UP_TO_TWO_PLACES);
        assert_eq!(value(ValueStyle::Short, 0.09), Precision::UP_TO_FOUR_PLACES);
        assert_eq!(value(ValueStyle::Auto, 1.0), Precision::UP_TO_TWO_PLACES);
        assert_eq!(value(ValueStyle::Auto, 0.99), Precision::FOUR_SIGNIFICANT);
    }

    #[test]
    fn test_only_the_short_style_abbreviates_or_reads_as_dust() {
        assert!(abbreviates_value(ValueStyle::Short, 100_000.0));
        assert!(!abbreviates_value(ValueStyle::Short, 99_999.0));
        assert!(!abbreviates_value(ValueStyle::Auto, 1_000_000.0));

        assert!(is_value_dust(ValueStyle::Short, 0.00009));
        assert!(!is_value_dust(ValueStyle::Short, 0.0001));
        assert!(!is_value_dust(ValueStyle::Short, 0.0));
        assert!(!is_value_dust(ValueStyle::Full, 0.00009));
    }

    #[test]
    fn test_dust_and_everything_from_one_up_reads_in_two_places() {
        assert_eq!(adaptive(0.0), Precision::TWO_PLACES);
        assert_eq!(adaptive(1e-11), Precision::TWO_PLACES);
        assert_eq!(adaptive(0.99), Precision::TWO_PLACES);
        assert_eq!(adaptive(1193.0109), Precision::TWO_PLACES);
    }
}
