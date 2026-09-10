use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::PaymentLink;

use crate::provider::PaymentProvider;
use crate::solana_pay::SolanaPayProvider;
use crate::wallet_connect_pay::{WalletConnectPayAuth, WalletConnectPayProvider};

pub(crate) struct PaymentProviderFactory {
    rpc_provider: Arc<dyn RpcProvider>,
    wallet_connect_pay_auth: WalletConnectPayAuth,
}

impl PaymentProviderFactory {
    pub(crate) fn new(rpc_provider: Arc<dyn RpcProvider>, wallet_connect_pay_auth: WalletConnectPayAuth) -> Self {
        Self {
            rpc_provider,
            wallet_connect_pay_auth,
        }
    }

    pub(crate) fn get_provider(&self, link: &PaymentLink) -> Box<dyn PaymentProvider> {
        match link {
            PaymentLink::SolanaPay { url } => Box::new(SolanaPayProvider::new(RpcClient::new(url.clone(), self.rpc_provider.clone()), url.clone())),
            PaymentLink::WalletConnectPay { payment_id } => Box::new(WalletConnectPayProvider::new(
                self.rpc_provider.clone(),
                self.wallet_connect_pay_auth.clone(),
                payment_id.to_string(),
            )),
        }
    }
}
