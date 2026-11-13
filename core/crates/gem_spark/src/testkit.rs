use crate::model::{LightningPayment, LightningPaymentDirection, LightningPaymentStatus};

pub const IDENTITY_PUBLIC_KEY: &str = "0281363910b0dc0015a4a25e758da30f0e28388ea5252c0e3713936f2d4ef7d3d5";
pub const ABANDON_SEED_HEX: &str = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";
pub const INVOICE_2000_SATS: &str = include_str!("../testdata/invoice_2000_sats.txt").trim_ascii();
pub const INVOICE_AMOUNTLESS: &str = include_str!("../testdata/invoice_amountless.txt").trim_ascii();

impl LightningPayment {
    pub fn mock(direction: LightningPaymentDirection, status: LightningPaymentStatus) -> Self {
        Self {
            id: "transfer".to_string(),
            direction,
            status,
            amount_sats: 1_000,
            fee_sats: 3,
            created_at: 1_792_993_669,
            invoice: None,
        }
    }
}
