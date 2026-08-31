use primitives::{AssetFiatValue, BalanceCalculator as Calculator, TotalFiatValue};

#[uniffi::remote(Record)]
pub struct AssetFiatValue {
    pub amount: f64,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[uniffi::remote(Record)]
pub struct TotalFiatValue {
    pub value: f64,
    pub pnl_amount: f64,
    pub pnl_percentage: f64,
}

#[derive(Debug, uniffi::Object)]
pub struct BalanceCalculator {}

impl Default for BalanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl BalanceCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn total_fiat_value(&self, balances: Vec<AssetFiatValue>) -> TotalFiatValue {
        Calculator::total_fiat_value(&balances)
    }

    pub fn wallet_total_fiat_value(&self, balances: Vec<AssetFiatValue>) -> TotalFiatValue {
        wallet_total_fiat_value(balances)
    }

    pub fn shows_pnl(&self, total: TotalFiatValue) -> bool {
        wallet_shows_pnl(total)
    }
}

fn wallet_total_fiat_value(balances: Vec<AssetFiatValue>) -> TotalFiatValue {
    let priced: Vec<AssetFiatValue> = balances.into_iter().filter(|balance| balance.price > 0.0).collect();
    Calculator::total_fiat_value(&priced)
}

fn wallet_shows_pnl(total: TotalFiatValue) -> bool {
    total.value > 0.0 && total.pnl_amount != 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(amount: f64, price: f64, change: f64) -> AssetFiatValue {
        AssetFiatValue {
            amount,
            price,
            price_change_percentage_24h: change,
        }
    }

    #[test]
    fn test_wallet_total_skips_assets_without_a_price() {
        let total = wallet_total_fiat_value(vec![value(2.0, 10.0, 0.0), value(100.0, 0.0, 5.0)]);

        assert_eq!(total.value, 20.0);
    }

    #[test]
    fn test_pnl_shows_only_for_a_funded_wallet_that_moved() {
        assert!(wallet_shows_pnl(TotalFiatValue {
            value: 20.0,
            pnl_amount: 1.0,
            pnl_percentage: 5.0
        }));
        assert!(!wallet_shows_pnl(TotalFiatValue {
            value: 20.0,
            pnl_amount: 0.0,
            pnl_percentage: 0.0
        }));
        assert!(!wallet_shows_pnl(TotalFiatValue {
            value: 0.0,
            pnl_amount: 1.0,
            pnl_percentage: 0.0
        }));
    }
}
