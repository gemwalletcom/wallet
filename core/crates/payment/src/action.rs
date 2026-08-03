use primitives::{ApprovalData, Chain, ChainAddress, PaymentQuote, PaymentQuotes, SignMessage, SignableTransaction};

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
    use primitives::TransferDataOutputType;

    fn send(chain: Chain) -> PaymentAction {
        PaymentAction::SendTransaction {
            chain,
            transaction: SignableTransaction::Ton {
                data: String::new(),
                output_type: TransferDataOutputType::EncodedTransaction,
            },
        }
    }

    #[test]
    fn test_validate_actions() {
        let addresses = vec![ChainAddress::new(Chain::Ethereum, "0x1".to_string())];

        assert!(validate_actions(&[send(Chain::Ethereum)], &addresses).is_ok());
        assert!(validate_actions(&[send(Chain::Ethereum), send(Chain::Solana)], &addresses).is_err());
        assert!(validate_actions(&[], &addresses).is_err());
    }
}
