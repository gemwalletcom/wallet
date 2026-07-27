use crate::{AssetFiatValue, PriceChangeCalculator, TotalFiatValue};

pub struct BalanceCalculator {}

impl BalanceCalculator {
    pub fn total_fiat_value(balances: &[AssetFiatValue]) -> TotalFiatValue {
        let (value, pnl_amount) = balances.iter().fold((0.0, 0.0), |(total, pnl), balance| {
            let fiat = balance.amount * balance.price;
            let amount = PriceChangeCalculator::amount(balance.price_change_percentage_24h, fiat);
            (total + fiat, pnl + amount)
        });
        TotalFiatValue {
            value,
            pnl_amount,
            pnl_percentage: PriceChangeCalculator::percentage(value - pnl_amount, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balance(amount: f64, price: f64, price_change_percentage_24h: f64) -> AssetFiatValue {
        AssetFiatValue {
            amount,
            price,
            price_change_percentage_24h,
        }
    }

    #[test]
    fn test_empty_balances() {
        let result = BalanceCalculator::total_fiat_value(&[]);
        assert_eq!(result.value, 0.0);
        assert_eq!(result.pnl_amount, 0.0);
        assert_eq!(result.pnl_percentage, 0.0);
    }

    #[test]
    fn test_positive_change() {
        let result = BalanceCalculator::total_fiat_value(&[balance(3.0, 1100.0, 10.0)]);
        assert_eq!(result.value, 3300.0);
        assert!((result.pnl_amount - 300.0).abs() < 1e-9);
        assert!((result.pnl_percentage - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_zero_change() {
        let result = BalanceCalculator::total_fiat_value(&[balance(1.0, 100.0, 0.0)]);
        assert_eq!(result.value, 100.0);
        assert_eq!(result.pnl_amount, 0.0);
        assert_eq!(result.pnl_percentage, 0.0);
    }

    #[test]
    fn test_mixed_balances_with_zero_pct_entry() {
        let result = BalanceCalculator::total_fiat_value(&[balance(3.0, 1100.0, 10.0), balance(500.0, 1.0, 0.0)]);
        assert_eq!(result.value, 3800.0);
        assert!((result.pnl_amount - 300.0).abs() < 1e-9);
        assert!((result.pnl_percentage - (300.0 / 3500.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn test_negative_change() {
        let result = BalanceCalculator::total_fiat_value(&[balance(1.0, 90.0, -10.0)]);
        assert_eq!(result.value, 90.0);
        assert!((result.pnl_amount - -10.0).abs() < 1e-9);
        assert!((result.pnl_percentage - -10.0).abs() < 1e-9);
    }
}
