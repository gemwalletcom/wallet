use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{Chain, ChainAddress, PaymentLink};

use crate::PaymentTransaction;
use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::solana_pay::SolanaPayProvider;

pub struct PaymentService {
    provider: Arc<dyn RpcProvider>,
}

impl PaymentService {
    pub fn new(provider: Arc<dyn RpcProvider>) -> Self {
        Self { provider }
    }

    pub async fn load(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentTransaction, PaymentError> {
        match link {
            PaymentLink::SolanaPay { url } => {
                let provider = SolanaPayProvider::new(RpcClient::new(url.clone(), self.provider.clone()), url.clone());
                Self::load_provider(&provider, addresses).await
            }
        }
    }

    async fn load_provider(provider: &impl PaymentProvider, addresses: &[ChainAddress]) -> Result<PaymentTransaction, PaymentError> {
        let transaction = provider.load(addresses).await?;
        Self::validate_account(provider.supported_chains(), &transaction.account, addresses)?;
        Ok(transaction)
    }

    fn validate_account(supported_chains: &[Chain], account: &ChainAddress, addresses: &[ChainAddress]) -> Result<(), PaymentError> {
        if !supported_chains.contains(&account.chain) {
            return Err(PaymentError::InvalidRequest {
                reason: "Payment account chain is not supported by provider".to_string(),
            });
        }
        if !addresses.contains(account) {
            return Err(PaymentError::InvalidRequest {
                reason: "Payment account changed".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_account() {
        let account = ChainAddress::new(Chain::Solana, "solana-account".to_string());

        assert_eq!(PaymentService::validate_account(&[Chain::Solana], &account, std::slice::from_ref(&account)), Ok(()));
        assert_eq!(
            PaymentService::validate_account(&[Chain::Ethereum], &account, std::slice::from_ref(&account)),
            Err(PaymentError::InvalidRequest {
                reason: "Payment account chain is not supported by provider".to_string(),
            })
        );
        assert_eq!(
            PaymentService::validate_account(&[Chain::Solana], &account, &[]),
            Err(PaymentError::InvalidRequest {
                reason: "Payment account changed".to_string(),
            })
        );
    }
}
