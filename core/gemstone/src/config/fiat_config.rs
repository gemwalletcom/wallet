#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct FiatConfig {
    pub default_buy_amount: i32,
    pub default_sell_amount: i32,
    pub minimum_amount: i32,
    pub maximum_amount: i32,
    pub random_max_amount: i32,
    pub suggested_amounts: Vec<i32>,
    pub insufficient_network_fee_buy_amount: i32,
}

pub fn get_fiat_config() -> FiatConfig {
    FiatConfig {
        default_buy_amount: 50,
        default_sell_amount: 100,
        minimum_amount: 5,
        maximum_amount: 10000,
        random_max_amount: 1000,
        suggested_amounts: vec![100, 250],
        insufficient_network_fee_buy_amount: 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fiat_config() {
        let config = get_fiat_config();

        assert_eq!(config.default_buy_amount, 50);
        assert_eq!(config.default_sell_amount, 100);
        assert_eq!(config.minimum_amount, 5);
        assert_eq!(config.maximum_amount, 10000);
        assert_eq!(config.random_max_amount, 1000);
        assert_eq!(config.suggested_amounts, vec![100, 250]);
        assert_eq!(config.insufficient_network_fee_buy_amount, 10);
    }
}
