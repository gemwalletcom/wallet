#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GasPriceType {
    Regular { gas_price: BigInt },
    Eip1559 {
        gas_price: BigInt,
        priority_fee: BigInt,
    },
    Legacy,
}

impl GasPriceType {
    pub fn gas_price(&self) -> BigInt {
        match self {
            Self::Regular { gas_price } | Self::Eip1559 { gas_price, .. } => gas_price.clone(),
            Self::Legacy => BigInt::from(0),
        }
    }
}
