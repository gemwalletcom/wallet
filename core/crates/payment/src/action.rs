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
    use primitives::{AssetId, Chain, PaymentAction, PaymentLink, PaymentQuote};

    fn payment(quoted: Chain, signing: Chain) -> PaymentQuoteData {
        PaymentQuoteData {
            quote: PaymentQuote {
                id: "opt_1".to_string(),
                link: PaymentLink::WalletConnectPay("pay_123".to_string()),
                asset_id: AssetId::from_chain(quoted),
                value: 1000u32.into(),
                expires_at: None,
                collect_data_url: None,
                provider_data: "{}".to_string(),
            },
            action: PaymentAction::Send {
                chain: signing,
                recipient: "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string(),
                value: 1000u32.into(),
                data: String::new(),
            },
        }
    }

    fn addresses(chains: &[Chain]) -> Vec<ChainAddress> {
        chains.iter().map(|chain| ChainAddress::new(*chain, "0x1".to_string())).collect()
    }

    #[test]
    fn test_validate_accepts_a_payment_signed_on_the_quoted_chain() {
        assert_eq!(validate(&payment(Chain::Ethereum, Chain::Ethereum), &addresses(&[Chain::Ethereum])), Ok(()));
    }

    #[test]
    fn test_validate_refuses_a_payment_signed_off_the_quoted_chain() {
        assert_eq!(
            validate(&payment(Chain::Ethereum, Chain::SmartChain), &addresses(&[Chain::Ethereum, Chain::SmartChain])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on smartchain for a quote on ethereum".to_string()))
        );
    }

    #[test]
    fn test_validate_refuses_a_payment_the_wallet_has_no_account_for() {
        assert_eq!(
            validate(&payment(Chain::Ethereum, Chain::Ethereum), &addresses(&[Chain::Bitcoin])),
            Err(PaymentError::InvalidRequest("Payment asks to sign on ethereum".to_string()))
        );
    }
}
