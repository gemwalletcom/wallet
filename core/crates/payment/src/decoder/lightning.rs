use num_bigint::BigUint;
use primitives::{
    AssetId, Chain,
    payment::{Payment, PaymentAmount, PaymentRequest},
};

use super::error::{PaymentDecoderError, Result};

pub fn decode(value: &str) -> Result<Payment> {
    let invoice = gem_spark::decode_invoice(value).map_err(|error| PaymentDecoderError::InvalidFormat(error.to_string()))?;
    Ok(Payment::Request {
        request: PaymentRequest {
            address: invoice.invoice,
            amount: invoice.amount_sats.map(|value| PaymentAmount::AtomicValue { value: BigUint::from(value) }),
            memo: None,
            label: None,
            references: None,
            asset_id: Some(AssetId::from_chain(Chain::Lightning)),
        },
    })
}
