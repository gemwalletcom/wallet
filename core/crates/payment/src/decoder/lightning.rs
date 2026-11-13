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

#[cfg(test)]
mod tests {
    use super::*;
    use gem_spark::testkit::{INVOICE_2000_SATS, INVOICE_AMOUNTLESS};

    fn request(value: &str) -> PaymentRequest {
        let Payment::Request { request } = decode(value).unwrap() else { panic!("expected a payment request") };
        request
    }

    #[test]
    fn test_decode() {
        let invoice = request(&format!("lightning:{}", INVOICE_2000_SATS.to_uppercase()));

        assert_eq!(invoice.address, INVOICE_2000_SATS);
        assert_eq!(invoice.amount, Some(PaymentAmount::AtomicValue { value: BigUint::from(2000u64) }));
        assert_eq!(invoice.asset_id, Some(AssetId::from_chain(Chain::Lightning)));
        assert_eq!(request(INVOICE_AMOUNTLESS).amount, None);
        assert!(decode("lnbc1invalid").is_err());
    }
}
