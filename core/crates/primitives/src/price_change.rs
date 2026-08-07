pub struct PriceChangeCalculator {}

impl PriceChangeCalculator {
    pub fn percentage(from: f64, to: f64) -> f64 {
        if from == 0.0 {
            return 0.0;
        }
        (to - from) / from * 100.0
    }

    pub fn amount(percentage: f64, value: f64) -> f64 {
        let denominator = 100.0 + percentage;
        if denominator == 0.0 {
            return 0.0;
        }
        value * percentage / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage() {
        assert_eq!(PriceChangeCalculator::percentage(100.0, 110.0), 10.0);
        assert_eq!(PriceChangeCalculator::percentage(100.0, 90.0), -10.0);
        assert_eq!(PriceChangeCalculator::percentage(200.0, 150.0), -25.0);
        assert_eq!(PriceChangeCalculator::percentage(50.0, 100.0), 100.0);
        assert_eq!(PriceChangeCalculator::percentage(100.0, 100.0), 0.0);
        assert_eq!(PriceChangeCalculator::percentage(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_amount() {
        assert_eq!(PriceChangeCalculator::amount(10.0, 110.0), 10.0);
        assert_eq!(PriceChangeCalculator::amount(-10.0, 90.0), -10.0);
        assert_eq!(PriceChangeCalculator::amount(0.0, 100.0), 0.0);
        assert_eq!(PriceChangeCalculator::amount(-100.0, 100.0), 0.0);
    }
}
