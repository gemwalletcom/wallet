#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Fraction { min: u32, max: u32 },
    Significant { max: u32 },
}

impl Precision {
    pub const TWO_PLACES: Self = Self::Fraction { min: 2, max: 2 };
    pub const UP_TO_TWO_PLACES: Self = Self::Fraction { min: 0, max: 2 };
    pub const UP_TO_FOUR_PLACES: Self = Self::Fraction { min: 0, max: 4 };
    pub const FOUR_SIGNIFICANT: Self = Self::Significant { max: 4 };
    pub const FULL: Self = Self::Fraction { min: 0, max: 32 };
}

pub const ABBREVIATION_THRESHOLD: f64 = 100_000.0;

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
    fn test_dust_and_everything_from_one_up_reads_in_two_places() {
        assert_eq!(adaptive(0.0), Precision::TWO_PLACES);
        assert_eq!(adaptive(1e-11), Precision::TWO_PLACES);
        assert_eq!(adaptive(0.99), Precision::TWO_PLACES);
        assert_eq!(adaptive(1193.0109), Precision::TWO_PLACES);
    }
}
