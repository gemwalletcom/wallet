use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{AssetId, Chain, ChainAddress, EVMChain, PaymentStatus, WalletConnectCAIP2};
use std::sync::{Arc, LazyLock};

use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::wallet_connect_pay::action_mapper::map_actions;
use crate::wallet_connect_pay::client::{WALLET_CONNECT_PAY_API_URL, WalletConnectPayClient};
use crate::wallet_connect_pay::config::WalletConnectPayAuth;
use crate::wallet_connect_pay::model::{Options, PaymentActions, Quote, WalletConnectPayAction, WalletRpcAction};
use crate::wallet_connect_pay::payment_mapper::{map_invoice, map_options, map_transaction};
use crate::{PaymentLoad, PaymentUpdate};

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
            Options::Status { status } => return Err(PaymentError::Status { status }),
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
                let url = invoice
                    .collect_data_url
                    .clone()
                    .or_else(|| quote.collect_data_url.clone())
                    .ok_or(PaymentError::invalid_request("Payment requires verification without a form"))?;
                return Ok(PaymentLoad::Verify {
                    invoice: payment_invoice,
                    asset_id: quote.asset_id.clone(),
                    url,
                });
            }
        };
        let actions: Vec<WalletRpcAction> = actions.into_iter().map(WalletRpcAction::try_from).collect::<Result<_, _>>()?;
        Ok(PaymentLoad::Sign {
            transaction: map_transaction(quote, map_actions(quote, &actions)?, payment_invoice),
        })
    }

    async fn get_actions(&self, quote: &Quote) -> Result<PaymentActions, PaymentError> {
        match quote.actions.first() {
            None => self.client.get_actions(&self.payment_id, &quote.id, String::new()).await,
            Some(WalletConnectPayAction::Build(build)) => self.client.get_actions(&self.payment_id, &quote.id, build.data.clone()).await,
            Some(WalletConnectPayAction::WalletRpc(_)) => Ok(PaymentActions::Ready(quote.actions.clone())),
        }
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
            status => Err(PaymentError::Status { status }),
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
    use crate::PaymentTransaction;
    use crate::wallet_connect_pay::payment_mapper::map_invoice;
    use crate::wallet_connect_pay::testkit;
    use crate::wallet_connect_pay::testkit::{
        FETCH_IDENTITY_REQUIRED, FETCH_SEND, OPTIONS, OPTIONS_FAILED, OPTIONS_IDENTITY_REQUIRED, STATUS_EXPIRED, STATUS_FAILED, STATUS_PROCESSING, STATUS_REQUIRES_ACTION,
        STATUS_SUCCEEDED, STATUS_SUCCEEDED_COIN, TEST_ACCOUNT, TEST_PAYMENT_ID, TEST_PERMIT_SPENDER, TEST_ROUTER, accounts, addresses, fetch_actions, options, quote,
        quote_actions, quotes,
    };
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::SMARTCHAIN_USDC_TOKEN_ID;
    use primitives::{PaymentAmount, PaymentInvoice, PaymentRequest, TransactionType, TransferDataOutputType};
    use serde_json::Value;

    const OPTIONS_PATH: &str = "/v1/gateway/payment/pay_6fa2ecc101M2NV05JCTQGE0FXR387X4TJB/options?includePaymentInfo=true";
    const FETCH_PATH: &str = "/v1/gateway/payment/pay_6fa2ecc101M2NV05JCTQGE0FXR387X4TJB/fetch";
    const CONFIRM_PATH: &str = "/v1/gateway/payment/pay_6fa2ecc101M2NV05JCTQGE0FXR387X4TJB/confirm";
    const STATUS_PATH: &str = "/v1/gateway/payment/pay_6fa2ecc101M2NV05JCTQGE0FXR387X4TJB/status?maxPollMs=0";

    fn provider(client: MockClient) -> WalletConnectPayProvider<MockClient> {
        let auth = WalletConnectPayAuth {
            app_id: "app".to_string(),
            client_id: "client".to_string(),
        };
        WalletConnectPayProvider {
            client: WalletConnectPayClient::new(client, auth),
            payment_id: TEST_PAYMENT_ID.to_string(),
        }
    }

    fn gateway(options: &'static str, fetch: Result<&'static str, ClientError>) -> MockClient {
        MockClient::new().with_post(move |path, _| match path {
            OPTIONS_PATH => Ok(options.as_bytes().to_vec()),
            FETCH_PATH => fetch.clone().map(|fetch| fetch.as_bytes().to_vec()),
            path => panic!("unexpected call {path}"),
        })
    }

    fn identity_required() -> ClientError {
        ClientError::Http {
            status: 400,
            body: FETCH_IDENTITY_REQUIRED.as_bytes().to_vec(),
        }
    }

    fn invoice(fixture: &str) -> PaymentInvoice {
        map_invoice(&testkit::invoice(options(fixture), &accounts(TEST_ACCOUNT)), TEST_PAYMENT_ID)
    }

    fn request(quote: &Quote, address: &str) -> Option<PaymentRequest> {
        Some(PaymentRequest {
            address: address.to_string(),
            amount: Some(PaymentAmount::AtomicValue { value: quote.value.clone() }),
            asset_id: Some(quote.asset_id.clone()),
            ..PaymentRequest::mock()
        })
    }

    #[tokio::test]
    async fn test_load() {
        let coin = quote(OPTIONS, TEST_ACCOUNT, &AssetId::from_chain(Chain::Optimism));
        let send = fetch_actions(FETCH_SEND);
        assert_eq!(
            provider(gateway(OPTIONS, Ok(FETCH_SEND))).load(&addresses(TEST_ACCOUNT)).await,
            Ok(PaymentLoad::Sign {
                transaction: PaymentTransaction {
                    invoice: invoice(OPTIONS),
                    account: coin.account.clone(),
                    transaction: send[0].params[0]["data"].as_str().unwrap().to_string(),
                    transaction_type: TransactionType::SmartContractCall,
                    memo: None,
                    request: request(&coin, TEST_ROUTER),
                    output_type: TransferDataOutputType::EncodedTransaction,
                    approval: None,
                }
            }),
            "the first quote, a coin, is built by the gateway"
        );
        assert_eq!(
            provider(gateway(OPTIONS_IDENTITY_REQUIRED, Err(identity_required()))).load(&addresses(TEST_ACCOUNT)).await,
            Ok(PaymentLoad::Verify {
                invoice: invoice(OPTIONS_IDENTITY_REQUIRED),
                asset_id: AssetId::from_chain(Chain::Optimism),
                url: quotes(OPTIONS_IDENTITY_REQUIRED, TEST_ACCOUNT)[0].collect_data_url.clone().unwrap().replace(
                    &format!("accounts=eip155%3A10%3A{TEST_ACCOUNT}"),
                    &format!(
                        "accounts={}",
                        ["42161", "10", "137", "8453", "1", "56"]
                            .map(|chain| format!("eip155%3A{chain}%3A{TEST_ACCOUNT}"))
                            .join("%2C")
                    ),
                ),
            }),
            "a refused quote verifies every account with the form of the response, not the one of the option"
        );
        assert_eq!(
            provider(gateway(OPTIONS_FAILED, Ok(FETCH_SEND))).load(&addresses(TEST_ACCOUNT)).await,
            Err(PaymentError::Status { status: PaymentStatus::Failed })
        );
        assert_eq!(
            provider(gateway(OPTIONS, Ok(FETCH_SEND)))
                .load(&[ChainAddress::new(Chain::Solana, TEST_ACCOUNT.to_string())])
                .await,
            Err(PaymentError::NoPaymentOptions),
            "no supported account asks the gateway nothing"
        );
    }

    #[tokio::test]
    async fn test_select_asset() {
        let usdc = AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID);
        let token = quote(OPTIONS, TEST_ACCOUNT, &usdc);
        let permit = quote_actions(&token);
        assert_eq!(
            provider(gateway(OPTIONS, Err(identity_required())))
                .select_asset(&addresses(TEST_ACCOUNT), usdc.clone())
                .await,
            Ok(PaymentLoad::Sign {
                transaction: PaymentTransaction {
                    invoice: invoice(OPTIONS),
                    account: token.account.clone(),
                    transaction: permit[0].params[1].as_str().unwrap().to_string(),
                    transaction_type: TransactionType::Transfer,
                    memo: None,
                    request: request(&token, TEST_PERMIT_SPENDER),
                    output_type: TransferDataOutputType::Signature,
                    approval: None,
                }
            }),
            "a token quote carries its actions, the gateway is not asked to build them"
        );
        assert_eq!(
            provider(gateway(OPTIONS, Ok(FETCH_SEND)))
                .select_asset(&addresses(TEST_ACCOUNT), AssetId::from_chain(Chain::Polygon))
                .await,
            Err(PaymentError::NoPaymentOptions)
        );
    }

    #[tokio::test]
    async fn test_confirm() {
        let gateway = |status: &'static str| {
            MockClient::new().with_post(move |path, body| match path {
                CONFIRM_PATH => {
                    assert_eq!(
                        serde_json::from_slice::<Value>(body).unwrap(),
                        serde_json::json!({
                            "optionId": "opt_1",
                            "results": [{"type": "walletRpc", "data": ["0xapprove"]}, {"type": "walletRpc", "data": ["0xsignature"]}]
                        }),
                        "one result per action, in the order of the actions"
                    );
                    Ok(status.as_bytes().to_vec())
                }
                path => panic!("unexpected call {path}"),
            })
        };
        let results = || vec!["0xapprove".to_string(), "0xsignature".to_string()];

        assert_eq!(provider(gateway(STATUS_PROCESSING)).confirm("opt_1", results()).await, Ok(()));
        assert_eq!(provider(gateway(STATUS_SUCCEEDED)).confirm("opt_1", results()).await, Ok(()));
        assert_eq!(
            provider(gateway(STATUS_FAILED)).confirm("opt_1", results()).await,
            Err(PaymentError::Status { status: PaymentStatus::Failed })
        );
        assert_eq!(
            provider(gateway(STATUS_REQUIRES_ACTION)).confirm("opt_1", results()).await,
            Err(PaymentError::Status {
                status: PaymentStatus::RequiresAction
            }),
            "results the gateway did not take leave the payment unpaid"
        );
    }

    #[tokio::test]
    async fn test_status() {
        let gateway = |status: &'static str| {
            MockClient::new().with_get(move |path| match path {
                STATUS_PATH => Ok(status.as_bytes().to_vec()),
                path => panic!("unexpected call {path}"),
            })
        };
        let update = |status: PaymentStatus, transaction_id: Option<&str>| {
            Ok(PaymentUpdate {
                status,
                transaction_id: transaction_id.map(str::to_string),
            })
        };

        assert_eq!(
            provider(gateway(STATUS_SUCCEEDED)).status().await,
            update(PaymentStatus::Succeeded, Some("0x46fc18618d378feb6cbdff6ba42231ccfb73eea417f666e43e8ecc7a88df8024")),
            "a relayed token payment reports the relayer's transaction"
        );
        assert_eq!(
            provider(gateway(STATUS_SUCCEEDED_COIN)).status().await,
            update(PaymentStatus::Succeeded, Some("0x068608e32ed1555d2955d791d8824a03b10b05db3bc7fd66784cde2196ba14b0")),
            "a coin payment reports the wallet's own transaction"
        );
        assert_eq!(provider(gateway(STATUS_REQUIRES_ACTION)).status().await, update(PaymentStatus::RequiresAction, None));
        assert_eq!(provider(gateway(STATUS_PROCESSING)).status().await, update(PaymentStatus::Processing, None));
        assert_eq!(provider(gateway(STATUS_FAILED)).status().await, update(PaymentStatus::Failed, None));
        assert_eq!(provider(gateway(STATUS_EXPIRED)).status().await, update(PaymentStatus::Expired, None));
    }
}
