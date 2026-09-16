use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{AssetId, Chain, ChainAddress, PaymentLink};

use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::solana_pay::SolanaPayProvider;
use crate::wallet_connect_pay::{WalletConnectPayAuth, WalletConnectPayProvider};
use crate::{PaymentLoad, PaymentUpdate};

pub struct PaymentService {
    rpc_provider: Arc<dyn RpcProvider>,
    wallet_connect_pay_auth: WalletConnectPayAuth,
}

impl PaymentService {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>, wallet_connect_pay_auth: WalletConnectPayAuth) -> Self {
        Self {
            rpc_provider,
            wallet_connect_pay_auth,
        }
    }

    pub async fn load(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        let provider = self.provider(link);
        Self::validate(provider.as_ref(), provider.load(addresses).await?, addresses)
    }

    pub async fn select_asset(&self, link: &PaymentLink, addresses: &[ChainAddress], asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        let provider = self.provider(link);
        Self::validate(provider.as_ref(), provider.select_asset(addresses, asset_id).await?, addresses)
    }

    pub async fn confirm(&self, link: &PaymentLink, quote_id: &str, action_results: Vec<String>) -> Result<(), PaymentError> {
        self.provider(link).confirm(quote_id, action_results).await
    }

    pub async fn status(&self, link: &PaymentLink) -> Result<PaymentUpdate, PaymentError> {
        self.provider(link).status().await
    }

    fn provider(&self, link: &PaymentLink) -> Box<dyn PaymentProvider> {
        match link {
            PaymentLink::SolanaPay { url } => Box::new(SolanaPayProvider::new(RpcClient::new(url.clone(), self.rpc_provider.clone()), url.clone())),
            PaymentLink::WalletConnectPay { payment_id } => Box::new(WalletConnectPayProvider::new(
                self.rpc_provider.clone(),
                self.wallet_connect_pay_auth.clone(),
                payment_id.clone(),
            )),
        }
    }

    fn validate(provider: &dyn PaymentProvider, load: PaymentLoad, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        if let Some(account) = load.account() {
            Self::validate_account(provider.supported_chains(), account, addresses)?;
        }
        Ok(load)
    }

    fn validate_account(supported_chains: &[Chain], account: &ChainAddress, addresses: &[ChainAddress]) -> Result<(), PaymentError> {
        if !supported_chains.contains(&account.chain) {
            return Err(PaymentError::invalid_request("Payment account chain is not supported by provider"));
        }
        if !addresses.contains(account) {
            return Err(PaymentError::invalid_request("Payment account changed"));
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
            Err(PaymentError::invalid_request("Payment account chain is not supported by provider"))
        );
        assert_eq!(
            PaymentService::validate_account(&[Chain::Solana], &account, &[]),
            Err(PaymentError::invalid_request("Payment account changed"))
        );
    }
}
