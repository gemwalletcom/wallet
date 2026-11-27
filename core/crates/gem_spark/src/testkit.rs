#[cfg(feature = "wallet")]
use std::sync::Mutex;

use date_time::DateTime;

#[cfg(feature = "wallet")]
use crate::error::LightningError;
use crate::model::{LightningPayment, LightningPaymentDirection, LightningPaymentStatus};
#[cfg(feature = "wallet")]
use crate::store::{LightningSend, LightningStore};

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
            created_at: DateTime::from_timestamp(1_792_993_669),
            invoice: None,
        }
    }
}

#[cfg(feature = "wallet")]
#[derive(Default)]
pub struct MemoryLightningStore {
    sends: Mutex<Vec<LightningSend>>,
}

#[cfg(feature = "wallet")]
impl LightningStore for MemoryLightningStore {
    fn get_sends(&self) -> Result<Vec<LightningSend>, LightningError> {
        Ok(self.sends.lock().unwrap().clone())
    }

    fn add_send(&self, send: LightningSend) -> Result<(), LightningError> {
        self.sends.lock().unwrap().push(send);
        Ok(())
    }

    fn delete_send(&self, transfer_id: &str) -> Result<(), LightningError> {
        self.sends.lock().unwrap().retain(|send| send.transfer_id != transfer_id);
        Ok(())
    }
}

#[cfg(feature = "wallet")]
impl LightningSend {
    pub fn mock(transfer_id: &str) -> Self {
        Self {
            transfer_id: transfer_id.to_string(),
            invoice: INVOICE_2000_SATS.to_string(),
            amount_sats: None,
        }
    }
}
