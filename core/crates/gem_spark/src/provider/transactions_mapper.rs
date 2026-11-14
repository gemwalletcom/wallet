use num_bigint::BigUint;
use primitives::{AssetId, Transaction, TransactionState, TransactionType};

use super::CHAIN;
use crate::model::{LightningPayment, LightningPaymentDirection, LightningPaymentStatus};

pub fn map_payment(address: &str, payment: &LightningPayment) -> Transaction {
    let asset_id = AssetId::from_chain(CHAIN);
    let (from, to) = match payment.direction {
        LightningPaymentDirection::Incoming => (String::new(), address.to_string()),
        LightningPaymentDirection::Outgoing => (address.to_string(), String::new()),
    };
    Transaction::new(
        payment.id.clone(),
        asset_id.clone(),
        from,
        to,
        None,
        TransactionType::Transfer,
        payment_state(payment.status),
        BigUint::from(payment.fee_sats),
        asset_id,
        BigUint::from(payment.amount_sats),
        None,
        None,
        payment.created_at,
    )
    .finalize(vec![address.to_string()])
}

pub(crate) fn payment_state(status: LightningPaymentStatus) -> TransactionState {
    match status {
        LightningPaymentStatus::Pending => TransactionState::Pending,
        LightningPaymentStatus::Completed => TransactionState::Confirmed,
        LightningPaymentStatus::Failed => TransactionState::Failed,
    }
}

#[cfg(test)]
mod tests {
    use primitives::TransactionDirection;

    use super::*;

    #[test]
    fn test_map_payment() {
        let incoming = map_payment("spark1address", &LightningPayment::mock(LightningPaymentDirection::Incoming, LightningPaymentStatus::Completed));
        let outgoing = map_payment("spark1address", &LightningPayment::mock(LightningPaymentDirection::Outgoing, LightningPaymentStatus::Pending));

        assert_eq!(incoming.direction, TransactionDirection::Incoming);
        assert_eq!((incoming.from.as_str(), incoming.to.as_str()), ("", "spark1address"));
        assert_eq!(incoming.state, TransactionState::Confirmed);
        assert_eq!(incoming.value, BigUint::from(1_000u64));
        assert_eq!(outgoing.direction, TransactionDirection::Outgoing);
        assert_eq!(outgoing.from, "spark1address");
        assert_eq!(outgoing.state, TransactionState::Pending);
        assert_eq!(outgoing.fee, BigUint::from(3u64));
    }
}
