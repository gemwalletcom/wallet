use primitives::{ChainAddress, PaymentQuoteData};

use crate::error::PaymentError;

pub fn validate(payment: &PaymentQuoteData, addresses: &[ChainAddress]) -> Result<(), PaymentError> {
    let chain = payment.action.chain();
    if chain != payment.quote.asset_id.chain {
        return Err(PaymentError::InvalidRequest(format!(
            "Payment asks to sign on {} for a quote on {}",
            chain.as_ref(),
            payment.quote.asset_id.chain.as_ref()
        )));
    }
    if !addresses.iter().any(|address| address.chain == chain) {
        return Err(PaymentError::InvalidRequest(format!("Payment asks to sign on {}", chain.as_ref())));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    fn addresses(chains: &[Chain]) -> Vec<ChainAddress> {
        chains.iter().map(|chain| ChainAddress::new(*chain, "0x1".to_string())).collect()
    }

    #[test]
    fn test_validate_accepts_a_payment_signed_on_the_quoted_chain() {
        let payment = PaymentQuoteData::mock(Chain::Ethereum, Chain::Ethereum);

        assert_eq!(validate(&payment, &addresses(&[Chain::Ethereum])), Ok(()));
    }

    #[test]
    fn test_validate_refuses_a_payment_signed_off_the_quoted_chain() {
        let payment = PaymentQuoteData::mock(Chain::Ethereum, Chain::SmartChain);

        assert_eq!(
            validate(&payment, &addresses(&[Chain::Ethereum, Chain::SmartChain])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on smartchain for a quote on ethereum".to_string()))
        );
    }

    #[test]
    fn test_validate_refuses_a_payment_the_wallet_has_no_account_for() {
        let payment = PaymentQuoteData::mock(Chain::Ethereum, Chain::Ethereum);

        assert_eq!(
            validate(&payment, &addresses(&[Chain::Bitcoin])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on ethereum".to_string()))
        );
    }
}
