use super::model::Hop;

impl Hop {
    pub fn mock() -> Self {
        Self {
            pool_id: "0xpool".into(),
            pool_init_version: 1,
            coin_a: "0xa".into(),
            coin_b: "0xb".into(),
            a2b: true,
            amount_in: 1_000,
            amount_out: 1_000_000,
            after_sqrt_price: 0,
        }
    }
}
