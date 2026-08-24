use primitives::{ChainAddress, PaymentQuoteData};

use crate::PaymentError;

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
    fn test_validate() {
        assert_eq!(validate(&PaymentQuoteData::mock(Chain::Solana, Chain::Solana), &addresses(&[Chain::Solana])), Ok(()));
        assert_eq!(
            validate(&PaymentQuoteData::mock(Chain::Solana, Chain::Ethereum), &addresses(&[Chain::Solana, Chain::Ethereum])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on ethereum for a quote on solana".to_string()))
        );
        assert_eq!(
            validate(&PaymentQuoteData::mock(Chain::Solana, Chain::Solana), &addresses(&[Chain::Bitcoin])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on solana".to_string()))
        );
    }
}
