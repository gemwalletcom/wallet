#[derive(Clone, Debug, PartialEq, Eq)]
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
