use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{AssetId, Chain, ChainAddress, EVMChain, PaymentStatus, WalletConnectCAIP2};
use std::sync::{Arc, LazyLock};

use crate::{PaymentLoad, PaymentUpdate};
use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::wallet_connect_pay::action_mapper::map_actions;
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
                let url = invoice.collect_data_url.clone().or_else(|| quote.collect_data_url.clone()).ok_or(PaymentError::InvalidRequest {
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
        map_actions(quote, &actions)
    }
}

#[async_trait]
impl<C: Client> PaymentProvider for WalletConnectPayProvider<C> {
    fn supported_chains(&self) -> &'static [Chain] {
        &SUPPORTED_CHAINS
    }

    async fn confirm(&self, quote_id: &str, action_results: Vec<String>) -> Result<(), PaymentError> {
        match self.client.confirm(&self.payment_id, quote_id, action_results).await?.status {
            PaymentStatus::Succeeded | PaymentStatus::Processing => Ok(()),
            status => Err(PaymentError::InvalidRequest { reason: status_reason(status) }),
        }
    }

    async fn status(&self) -> Result<PaymentUpdate, PaymentError> {
        let response = self.client.get_status(&self.payment_id).await?;
        Ok(PaymentUpdate {
            status: response.status,
            transaction_id: response.info.and_then(|info| info.tx_id),
        })
    }

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentLoad, PaymentError> {
        self.load_quote(addresses, None).await
    }

    async fn select_asset(&self, addresses: &[ChainAddress], asset_id: AssetId) -> Result<PaymentLoad, PaymentError> {
        self.load_quote(addresses, Some(asset_id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    fn provider(status: &'static str) -> WalletConnectPayProvider<MockClient> {
        let client = MockClient::new().with_get(move |path| match path {
            "/v1/gateway/payment/pay_1/status?maxPollMs=0" => Ok(status.as_bytes().to_vec()),
            path => panic!("unexpected call {path}"),
        });
        WalletConnectPayProvider {
            client: WalletConnectPayClient::new(
                client,
                WalletConnectPayAuth {
                    app_id: "app".to_string(),
                    client_id: "client".to_string(),
                },
            ),
            payment_id: "pay_1".to_string(),
        }
    }

    #[tokio::test]
    async fn test_a_refused_option_verifies_every_account_with_one_form() {
        let options = r#"{"info":{"status":"requires_action","merchant":{"name":"Gem Coffee"},"amount":{"unit":"iso4217/USD","value":"10","display":{"decimals":2}}},
            "collectData":{"url":"https://pay.walletconnect.com/collect/?pid=pay_1&accounts=eip155:10:0xa,eip155:56:0xa"},
            "options":[{"id":"opt_op","account":"eip155:10:0x1085c5f70F7F7591D97da281A64688385455c2bD","amount":{"unit":"caip19/eip155:10/slip44:60","value":"1000"},
                        "collectData":{"url":"https://pay.walletconnect.com/collect/?pid=pay_1&accounts=eip155:10:0xa"}}]}"#;
        let client = MockClient::new().with_post(move |path, _| match path {
            "/v1/gateway/payment/pay_1/options?includePaymentInfo=true" => Ok(options.as_bytes().to_vec()),
            "/v1/gateway/payment/pay_1/fetch" => Err(gem_client::ClientError::Http {
                status: 400,
                body: br#"{"code":"params_validation","message":"IC data required but not found"}"#.to_vec(),
            }),
            path => panic!("unexpected call {path}"),
        });
        let provider = WalletConnectPayProvider {
            client: WalletConnectPayClient::new(
                client,
                WalletConnectPayAuth {
                    app_id: "app".to_string(),
                    client_id: "client".to_string(),
                },
            ),
            payment_id: "pay_1".to_string(),
        };

        let PaymentLoad::Verify { url, asset_id, .. } = provider
            .load(&[ChainAddress::new(Chain::Optimism, "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string())])
            .await
            .unwrap()
        else {
            panic!("expected a verification");
        };
        assert_eq!(url, "https://pay.walletconnect.com/collect/?pid=pay_1&accounts=eip155:10:0xa,eip155:56:0xa");
        assert_eq!(asset_id, AssetId::from_chain(Chain::Optimism));
    }

    #[tokio::test]
    async fn test_status_reports_the_relayed_transaction() {
        let succeeded = r#"{"status":"succeeded","isFinal":true,"info":{"txId":"0xrelayed","optionAmount":{"value":"1000","unit":"caip19/eip155:137/slip44:966"}}}"#;
        let processing = r#"{"status":"processing","isFinal":false,"pollInMs":2000}"#;
        let bare = r#"{"status":"succeeded","info":null}"#;

        assert_eq!(
            provider(succeeded).status().await,
            Ok(PaymentUpdate {
                status: PaymentStatus::Succeeded,
                transaction_id: Some("0xrelayed".to_string()),
            })
        );
        assert_eq!(
            provider(processing).status().await,
            Ok(PaymentUpdate {
                status: PaymentStatus::Processing,
                transaction_id: None,
            })
        );
        assert_eq!(provider(bare).status().await.unwrap().transaction_id, None);
    }
}
