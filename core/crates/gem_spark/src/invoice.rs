use std::str::FromStr;

use lightning_invoice::{Bolt11Invoice, Currency};

use crate::error::LightningError;
use crate::model::LightningInvoice;

pub const LIGHTNING_SCHEME: &str = "lightning:";
const MSATS_PER_SAT: u64 = 1_000;

pub fn decode_invoice(value: &str) -> Result<LightningInvoice, LightningError> {
    let trimmed = value.trim();
    let invoice = match trimmed.get(..LIGHTNING_SCHEME.len()) {
        Some(prefix) if prefix.eq_ignore_ascii_case(LIGHTNING_SCHEME) => &trimmed[LIGHTNING_SCHEME.len()..],
        _ => trimmed,
    }
    .to_ascii_lowercase();
    let decoded = Bolt11Invoice::from_str(&invoice).map_err(|error| LightningError::InvalidInvoice(error.to_string()))?;
    if decoded.currency() != Currency::Bitcoin {
        return Err(LightningError::InvoiceNetworkMismatch);
    }
    let created_at = decoded.duration_since_epoch().as_secs();
    Ok(LightningInvoice {
        amount_sats: decoded.amount_milli_satoshis().map(|amount| amount.div_ceil(MSATS_PER_SAT)),
        payment_hash: decoded.payment_hash().to_string(),
        created_at,
        expires_at: created_at.saturating_add(decoded.expiry_time().as_secs()),
        invoice,
    })
}

#[cfg(any(feature = "wallet", test))]
pub fn requested_amount(invoice: &LightningInvoice, amount_sats: u64) -> Option<u64> {
    match invoice.amount_sats {
        Some(_) => None,
        None => Some(amount_sats),
    }
}

#[cfg(any(feature = "wallet", test))]
pub fn send_amount(invoice: &LightningInvoice, requested: u64, now: u64) -> Result<u64, LightningError> {
    if invoice.expires_at <= now {
        return Err(LightningError::InvoiceExpired);
    }
    match invoice.amount_sats {
        Some(amount) if amount == requested => Ok(amount),
        Some(amount) => Err(LightningError::AmountMismatch { invoice: amount, requested }),
        None if requested > 0 => Ok(requested),
        None => Err(LightningError::AmountRequired),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{INVOICE_2000_SATS, INVOICE_AMOUNTLESS};

    #[test]
    fn test_decode_invoice() {
        let invoice = decode_invoice(INVOICE_2000_SATS).unwrap();
        assert_eq!(
            invoice,
            LightningInvoice {
                invoice: INVOICE_2000_SATS.to_string(),
                amount_sats: Some(2000),
                payment_hash: "386b1e32b89eaf4f4d4a154cffa8fe5c2cd06558294da14d6600b3afaa6d0cc1".to_string(),
                created_at: 1_792_993_669,
                expires_at: 1_792_997_269,
            }
        );

        assert_eq!(decode_invoice(&format!("LIGHTNING:{}", INVOICE_2000_SATS.to_uppercase())).unwrap(), invoice);
        assert_eq!(decode_invoice(INVOICE_AMOUNTLESS).unwrap().amount_sats, None);
        assert!(matches!(decode_invoice("lnbc1invalid"), Err(LightningError::InvalidInvoice(_))));
    }

    #[test]
    fn test_requested_amount() {
        assert_eq!(requested_amount(&decode_invoice(INVOICE_2000_SATS).unwrap(), 1), None);
        assert_eq!(requested_amount(&decode_invoice(INVOICE_AMOUNTLESS).unwrap(), 500), Some(500));
    }

    #[test]
    fn test_send_amount() {
        let invoice = decode_invoice(INVOICE_2000_SATS).unwrap();
        let amountless = decode_invoice(INVOICE_AMOUNTLESS).unwrap();
        let now = invoice.created_at;

        assert_eq!(send_amount(&invoice, 2000, now), Ok(2000));
        assert_eq!(send_amount(&invoice, 1000, now), Err(LightningError::AmountMismatch { invoice: 2000, requested: 1000 }));
        assert_eq!(send_amount(&invoice, 2000, invoice.expires_at), Err(LightningError::InvoiceExpired));

        let now = amountless.created_at;
        assert_eq!(send_amount(&amountless, 500, now), Ok(500));
        assert_eq!(send_amount(&amountless, 0, now), Err(LightningError::AmountRequired));
    }
}
