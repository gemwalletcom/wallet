use std::sync::Arc;

use gem_jsonrpc::alien::RpcProvider;
use primitives::{AssetId, Chain, ChainAddress, PaymentLink};

use crate::PaymentLoad;
use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::provider_factory::PaymentProviderFactory;
use crate::wallet_connect_pay::WalletConnectPayAuth;

pub struct PaymentService {
    providers: PaymentProviderFactory,
}

impl PaymentService {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>, wallet_connect_pay_auth: WalletConnectPayAuth) -> Self {
        Self {
            providers: PaymentProviderFactory::new(rpc_provider, wallet_connect_pay_auth),
        }
    }

    pub async fn load(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        let provider = self.providers.get_provider(link);
        Self::validate(provider.as_ref(), provider.load(addresses).await?, addresses)
    }

    pub async fn select_asset(&self, link: &PaymentLink, addresses: &[ChainAddress], asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        let provider = self.providers.get_provider(link);
        Self::validate(provider.as_ref(), provider.select_asset(addresses, asset_id).await?, addresses)
    }

    pub async fn confirm(&self, link: &PaymentLink, quote_id: &str, transaction_hash: String) -> Result<(), PaymentError> {
        self.providers.get_provider(link).confirm(quote_id, transaction_hash).await
    }

    fn validate(provider: &dyn PaymentProvider, load: PaymentLoad, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        if let Some(account) = load.account() {
            Self::validate_account(provider.supported_chains(), account, addresses)?;
        }
        Ok(load)
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
