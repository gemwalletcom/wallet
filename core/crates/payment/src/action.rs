use primitives::swap::ApprovalData;
use primitives::{Chain, ChainAddress, PaymentQuote, PaymentQuotes, SignMessage, SignableTransaction};

use crate::error::PaymentError;

#[derive(Debug)]
pub enum PaymentAction {
    SignMessage { message: SignMessage },
    SignTransaction { chain: Chain, transaction: SignableTransaction },
    SendTransaction { chain: Chain, transaction: SignableTransaction },
    ApproveToken { chain: Chain, approval: ApprovalData },
}

impl PaymentAction {
    pub fn chain(&self) -> Chain {
        match self {
            Self::SignMessage { message } => message.chain,
            Self::SignTransaction { chain, .. } | Self::SendTransaction { chain, .. } | Self::ApproveToken { chain, .. } => *chain,
        }
    }
}

#[derive(Debug)]
pub struct PreparedPayment {
    pub quotes: PaymentQuotes,
    pub quote: PaymentQuote,
    pub actions: Vec<PaymentAction>,
}

impl PreparedPayment {
    pub fn validate(&self, addresses: &[ChainAddress]) -> Result<(), PaymentError> {
        validate_actions(&self.actions, addresses)
    }

    pub fn is_relayed(&self) -> bool {
        !self.actions.iter().any(|action| matches!(action, PaymentAction::SendTransaction { .. }))
    }
}

fn validate_actions(actions: &[PaymentAction], addresses: &[ChainAddress]) -> Result<(), PaymentError> {
    if actions.is_empty() {
        return Err(PaymentError::InvalidRequest("Payment has no actions".to_string()));
    }
    match actions.iter().find(|action| !addresses.iter().any(|address| address.chain == action.chain())) {
        Some(action) => Err(PaymentError::InvalidRequest(format!("Payment asks to sign on {}", action.chain().as_ref()))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, PaymentAmount, PaymentMerchant, TransferDataOutputType};

    fn send(chain: Chain) -> PaymentAction {
        PaymentAction::SendTransaction {
            chain,
            transaction: SignableTransaction::Ton {
                data: String::new(),
                output_type: TransferDataOutputType::EncodedTransaction,
            },
        }
    }

    fn sign(chain: Chain) -> PaymentAction {
        PaymentAction::SignTransaction {
            chain,
            transaction: SignableTransaction::Ton {
                data: String::new(),
                output_type: TransferDataOutputType::Signature,
            },
        }
    }

    fn prepared(actions: Vec<PaymentAction>) -> PreparedPayment {
        PreparedPayment {
            quotes: PaymentQuotes {
                merchant: PaymentMerchant {
                    name: "Merchant".to_string(),
                    icon_url: None,
                },
                price: None,
                expires_at: None,
                quotes: vec![],
            },
            quote: PaymentQuote {
                id: "option_1".to_string(),
                payment_id: "pay_1".to_string(),
                amount: PaymentAmount {
                    asset_id: AssetId::from_chain(Chain::Ethereum),
                    value: "1".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                },
                expires_at: None,
                collect_data_url: None,
                provider_data: "{}".to_string(),
            },
            actions,
        }
    }

    #[test]
    fn test_is_relayed() {
        assert!(prepared(vec![sign(Chain::Ethereum)]).is_relayed());
        assert!(prepared(vec![]).is_relayed());
        assert!(!prepared(vec![send(Chain::Ethereum)]).is_relayed());
        assert!(!prepared(vec![sign(Chain::Ethereum), send(Chain::Ethereum)]).is_relayed());
    }

    #[test]
    fn test_validate_actions() {
        let addresses = vec![ChainAddress::new(Chain::Ethereum, "0x1".to_string())];

        assert!(validate_actions(&[send(Chain::Ethereum)], &addresses).is_ok());
        assert!(validate_actions(&[send(Chain::Ethereum), send(Chain::Solana)], &addresses).is_err());
        assert!(validate_actions(&[], &addresses).is_err());
    }
}
