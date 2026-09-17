use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FeemarketGasPriceResponse {
    pub price: FeemarketGasPrice,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeemarketGasPrice {
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BaseFeeResponse {
    pub base_fee: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InjectiveBaseFeeResponse {
    pub base_fee: BaseFeeResponse,
}
