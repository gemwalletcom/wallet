use crate::{AssetFiatValue, AssetId, Chain};

impl AssetFiatValue {
    pub fn mock(amount: f64, price: f64, price_change_percentage_24h: f64) -> Self {
        AssetFiatValue {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            amount,
            price,
            price_change_percentage_24h,
        }
    }
}
