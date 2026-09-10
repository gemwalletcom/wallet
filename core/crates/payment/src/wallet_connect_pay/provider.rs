use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{AssetId, Chain, ChainAddress, EVMChain, PaymentStatus, WalletConnectCAIP2};
use std::sync::{Arc, LazyLock};

use crate::PaymentLoad;
use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::wallet_connect_pay::action_mapper::map_wallet_rpc;
use crate::wallet_connect_pay::client::{WALLET_CONNECT_PAY_API_URL, WalletConnectPayClient};
use crate::wallet_connect_pay::config::WalletConnectPayAuth;
use crate::wallet_connect_pay::model::{Options, PaymentAction, PaymentActions, Quote, WalletConnectPayAction, WalletRpcAction};
use crate::wallet_connect_pay::payment_mapper::{map_invoice, map_options, map_transaction, status_reason};

static SUPPORTED_CHAINS: LazyLock<Vec<Chain>> = LazyLock::new(|| EVMChain::all().into_iter().map(|chain| chain.to_chain()).collect());

#[derive(Debug)]
pub(crate) struct WalletConnectPayProvider<C: Client> {
    client: WalletConnectPayClient<C>,
    payment_id: String,
}

impl WalletConnectPayProvider<RpcClient> {
    pub(crate) fn new(rpc_provider: Arc<dyn RpcProvider>, auth: WalletConnectPayAuth, payment_id: String) -> Self {
        let client = WalletConnectPayClient::new(RpcClient::new(WALLET_CONNECT_PAY_API_URL.to_string(), rpc_provider), auth);
        Self { client, payment_id }
    }
}

impl<C: Client> WalletConnectPayProvider<C> {
    async fn get_options(&self, addresses: &[ChainAddress]) -> Result<Options, PaymentError> {
        let accounts: Vec<String> = addresses
            .iter()
            .filter(|address| SUPPORTED_CHAINS.contains(&address.chain))
            .filter_map(|address| WalletConnectCAIP2::format_account(address.chain, &address.address))
            .collect();
        if accounts.is_empty() {
            return Err(PaymentError::NoPaymentOptions);
        }
        let response = self.client.get_options(&self.payment_id, &accounts).await?;
        map_options(response, &accounts)
    }

    async fn load_quote(&self, addresses: &[ChainAddress], asset_id: Option<AssetId>) -> Result<PaymentLoad, PaymentError> {
        let invoice = match self.get_options(addresses).await? {
            Options::Status { status } => return Err(PaymentError::InvalidRequest { reason: status_reason(status) }),
            Options::Invoice(invoice) => invoice,
        };
        let quote = match &asset_id {
            Some(asset_id) => invoice.quotes.iter().find(|quote| &quote.asset_id == asset_id),
            None => invoice.quotes.first(),
        }
        .ok_or(PaymentError::NoPaymentOptions)?;
        let payment_invoice = map_invoice(&invoice, &self.payment_id);

        let actions = match self.get_actions(quote).await? {
            PaymentActions::Ready(actions) => actions,
            PaymentActions::CollectData => {
                let url = quote.collect_data_url.clone().ok_or(PaymentError::InvalidRequest {
                    reason: "Payment requires verification without a form".to_string(),
                })?;
                return Ok(PaymentLoad::Verify {
                    invoice: payment_invoice,
                    asset_id: quote.asset_id.clone(),
                    url,
                });
            }
        };
        Ok(PaymentLoad::Sign {
            transaction: map_transaction(quote, Self::get_action(quote, actions)?, payment_invoice),
        })
    }

    async fn get_actions(&self, quote: &Quote) -> Result<PaymentActions, PaymentError> {
        match quote.actions.first() {
            None => self.client.get_actions(&self.payment_id, &quote.id, String::new()).await,
            Some(WalletConnectPayAction::Build(build)) => self.client.get_actions(&self.payment_id, &quote.id, build.data.clone()).await,
            Some(WalletConnectPayAction::WalletRpc(_)) => Ok(PaymentActions::Ready(quote.actions.clone())),
        }
    }

    fn get_action(quote: &Quote, actions: Vec<WalletConnectPayAction>) -> Result<PaymentAction, PaymentError> {
        let actions: Vec<WalletRpcAction> = actions.into_iter().map(WalletRpcAction::try_from).collect::<Result<_, _>>()?;
        match actions.as_slice() {
            [action] => map_wallet_rpc(&quote.account, &quote.value, action),
            actions => Err(PaymentError::InvalidRequest {
                reason: format!("Payment asks for {} actions", actions.len()),
            }),
        }
    }
}

#[async_trait]
impl<C: Client> PaymentProvider for WalletConnectPayProvider<C> {
    fn supported_chains(&self) -> &'static [Chain] {
        &SUPPORTED_CHAINS
    }

    async fn confirm(&self, quote_id: &str, transaction_hash: String) -> Result<(), PaymentError> {
        match self.client.confirm(&self.payment_id, quote_id, transaction_hash).await?.status {
            PaymentStatus::Succeeded | PaymentStatus::Processing => Ok(()),
            status => Err(PaymentError::InvalidRequest { reason: status_reason(status) }),
        }
    }

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        self.load_quote(addresses, None).await
    }

    async fn select_asset(&self, addresses: &[ChainAddress], asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        self.load_quote(addresses, Some(asset_id)).await
    }
}
