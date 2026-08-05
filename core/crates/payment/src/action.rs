use primitives::swap::ApprovalData;
use primitives::{AssetId, Chain, ChainAddress, PaymentQuote, PaymentQuotes, SignMessage, SignableTransaction};

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
        validate_actions(&self.actions, addresses)?;
        validate_approvals(&self.actions, &self.quote.amount.asset_id)
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

fn validate_approvals(actions: &[PaymentAction], asset_id: &AssetId) -> Result<(), PaymentError> {
    for action in actions {
        if let PaymentAction::ApproveToken { chain, .. } = action
            && *chain != asset_id.chain
        {
            return Err(PaymentError::InvalidRequest(format!(
                "Payment asks to approve on {} for an asset on {}",
                chain.as_ref(),
                asset_id.chain.as_ref()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{PaymentAmount, PaymentMerchant, TransferDataOutputType};

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

    fn approve(chain: Chain) -> PaymentAction {
        PaymentAction::ApproveToken {
            chain,
            approval: ApprovalData {
                token: "0xtoken".to_string(),
                spender: "0xspender".to_string(),
                value: "1".to_string(),
                is_unlimited: false,
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
    fn test_validate_approvals() {
        let addresses = vec![
            ChainAddress::new(Chain::Ethereum, "0x1".to_string()),
            ChainAddress::new(Chain::Polygon, "0x1".to_string()),
        ];

        assert!(prepared(vec![approve(Chain::Ethereum)]).validate(&addresses).is_ok());
        assert!(prepared(vec![approve(Chain::Polygon)]).validate(&addresses).is_err());
        assert!(prepared(vec![approve(Chain::Ethereum), send(Chain::Ethereum)]).validate(&addresses).is_ok());
    }

    #[test]
    fn test_validate_actions() {
        let addresses = vec![ChainAddress::new(Chain::Ethereum, "0x1".to_string())];

        assert!(validate_actions(&[send(Chain::Ethereum)], &addresses).is_ok());
        assert!(validate_actions(&[send(Chain::Ethereum), send(Chain::Solana)], &addresses).is_err());
        assert!(validate_actions(&[], &addresses).is_err());
    }
}
