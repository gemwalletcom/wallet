use crate::wallet_connect_pay::account::account_identifier;
use crate::wallet_connect_pay::action_mapper::map_wallet_rpc;
use chrono::Utc;
use gem_client::Client;
use primitives::{AssetId, ChainAddress, PaymentOptions, PaymentOutcome, PaymentQuote, PaymentQuotes, PaymentStatus};

use crate::error::PaymentError;
use crate::wallet_connect_pay::client::{WalletConnectPayAuth, WalletConnectPayClient};
use crate::wallet_connect_pay::model::{PaymentOption, WalletConnectPayAction, WalletRpcAction};
use crate::wallet_connect_pay::payment_mapper;
use crate::wallet_connect_pay::quote::QuotedOption;
use crate::{PaymentAction, PreparedPayment};
use primitives::payment_decoder::wallet_connect_pay::{WALLET_CONNECT_HOST, is_wallet_connect_url};

#[derive(Debug)]
pub struct WalletConnectPayService<C: Client> {
    client: WalletConnectPayClient<C>,
}

impl<C: Client> WalletConnectPayService<C> {
    pub fn new(client: C, auth: WalletConnectPayAuth) -> Self {
        Self {
            client: WalletConnectPayClient::new(client, auth),
        }
    }

    pub(crate) async fn prepare_payment(&self, quotes: &PaymentQuotes, quote: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PreparedPayment, PaymentError> {
        if Self::is_expired(quote) {
            return self.get_requoted_actions(quote, addresses).await;
        }
        match self.payment_actions(quote).await {
            Ok(actions) => Ok(PreparedPayment {
                quotes: quotes.clone(),
                quote: quote.clone(),
                actions,
            }),
            Err(PaymentError::QuoteExpired) => self.get_requoted_actions(quote, addresses).await,
            Err(error) => Err(error),
        }
    }

    async fn get_requoted_actions(&self, expired: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PreparedPayment, PaymentError> {
        let quotes = match self.payment_options(&expired.payment_id, addresses).await? {
            PaymentOptions::Quotes(quotes) => quotes,
            PaymentOptions::Outcome(_) => return Err(PaymentError::PaymentExpired),
        };
        let quote = Self::get_quote(&quotes, &expired.amount.asset_id)?;
        let actions = self.payment_actions(&quote).await?;
        Ok(PreparedPayment { quotes, quote, actions })
    }

    pub(crate) async fn confirm_payment(&self, quote: &PaymentQuote, action_results: Vec<String>) -> Result<PaymentOutcome, PaymentError> {
        let response = self.client.confirm(&quote.payment_id, &quote.id, action_results).await?;
        Ok(payment_mapper::map_payment_outcome(response))
    }

    pub(crate) async fn payment_options(&self, payment_id: &str, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        let identifiers: Vec<String> = addresses.iter().filter_map(|address| account_identifier(address.chain, &address.address)).collect();
        if identifiers.is_empty() {
            return Err(PaymentError::UnsupportedAccounts);
        }
        let response = self.client.get_options(payment_id, identifiers).await?;
        let quoted = payment_mapper::map_quoted_payment(response)?;

        match quoted.status {
            PaymentStatus::RequiresAction => {}
            PaymentStatus::Succeeded | PaymentStatus::Processing => {
                return Ok(PaymentOptions::Outcome(PaymentOutcome {
                    status: quoted.status,
                    transaction_id: None,
                }));
            }
            PaymentStatus::Failed | PaymentStatus::Expired | PaymentStatus::Cancelled => return Err(PaymentError::PaymentExpired),
        }
        if quoted.expires_at <= Utc::now() {
            return Err(PaymentError::PaymentExpired);
        }

        let quotes = Self::payment_quotes(payment_id, quoted.options)?;
        if quotes.is_empty() {
            return Err(PaymentError::NoPaymentOptions);
        }

        Ok(PaymentOptions::Quotes(PaymentQuotes {
            merchant: quoted.merchant,
            price: Some(quoted.price),
            expires_at: Some(quoted.expires_at),
            quotes,
        }))
    }

    fn is_expired(quote: &PaymentQuote) -> bool {
        quote.expires_at.is_some_and(|expires_at| expires_at <= Utc::now())
    }

    fn get_quote(quotes: &PaymentQuotes, asset_id: &AssetId) -> Result<PaymentQuote, PaymentError> {
        quotes
            .quotes
            .iter()
            .find(|quote| &quote.amount.asset_id == asset_id)
            .cloned()
            .ok_or(PaymentError::QuoteExpired)
    }

    async fn payment_actions(&self, quote: &PaymentQuote) -> Result<Vec<PaymentAction>, PaymentError> {
        let option = Self::option(quote)?;
        let actions = self.wallet_rpc_actions(&quote.payment_id, &option).await?;
        if actions.is_empty() {
            return Err(PaymentError::InvalidRequest("Payment option has no executable actions".to_string()));
        }

        actions.iter().map(|action| map_wallet_rpc(&option.account, action)).collect()
    }

    pub(crate) async fn cancel_payment(&self, payment_id: &str) -> Result<(), PaymentError> {
        self.client.cancel(payment_id).await
    }

    pub(crate) async fn get_payment_status(&self, payment_id: &str) -> Result<PaymentOutcome, PaymentError> {
        let response = self.client.get_status(payment_id).await?;
        Ok(payment_mapper::map_payment_outcome(response))
    }

    fn payment_quotes(payment_id: &str, options: Vec<QuotedOption>) -> Result<Vec<PaymentQuote>, PaymentError> {
        let (open, collecting): (Vec<QuotedOption>, Vec<QuotedOption>) = options.into_iter().partition(|option| option.collect_data_url.is_none());
        open.into_iter()
            .chain(collecting)
            .map(|option| {
                if let Some(url) = &option.collect_data_url
                    && !is_wallet_connect_url(url)
                {
                    return Err(PaymentError::InvalidRequest(format!("Payment collects data outside {WALLET_CONNECT_HOST}")));
                }
                Ok(PaymentQuote {
                    id: option.id,
                    payment_id: payment_id.to_string(),
                    amount: option.amount,
                    expires_at: option.expires_at,
                    collect_data_url: option.collect_data_url,
                    provider_data: option.provider_data,
                })
            })
            .collect()
    }

    async fn wallet_rpc_actions(&self, payment_id: &str, option: &PaymentOption) -> Result<Vec<WalletRpcAction>, PaymentError> {
        let actions = match option.actions.first() {
            None => self.client.get_actions(payment_id, &option.id, String::new()).await?,
            Some(WalletConnectPayAction::Build(build)) => self.client.get_actions(payment_id, &option.id, build.data.clone()).await?,
            Some(WalletConnectPayAction::WalletRpc(_)) => option.actions.clone(),
        };

        actions.into_iter().map(WalletRpcAction::try_from).collect()
    }

    fn option(quote: &PaymentQuote) -> Result<PaymentOption, PaymentError> {
        serde_json::from_str(&quote.provider_data).map_err(|error| PaymentError::InvalidRequest(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use primitives::{AssetId, Chain, PaymentAmount, PaymentMerchant};
    use std::sync::{Arc, Mutex};

    fn service(mock: MockClient) -> WalletConnectPayService<MockClient> {
        WalletConnectPayService::new(mock, WalletConnectPayAuth::mock())
    }

    fn service_with_response(transform: impl Fn(&mut serde_json::Value) + Send + Sync + 'static) -> WalletConnectPayService<MockClient> {
        service(MockClient::new().with_post(move |_, _| {
            let mut response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/options_response.json")).unwrap();
            transform(&mut response);
            Ok(response.to_string().into_bytes())
        }))
    }

    #[tokio::test]
    async fn test_prepared_payment_requotes_a_stale_quote() {
        let requested = Arc::new(Mutex::new(Vec::<String>::new()));
        let seen = requested.clone();
        let client = MockClient::new().with_post(move |path: &str, _| {
            seen.lock().unwrap().push(path.to_string());
            if path.contains("/fetch") {
                return Err(ClientError::Http {
                    status: 409,
                    body: br#"{"code":"quote_expired"}"#.to_vec(),
                });
            }
            let mut response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/options_response.json")).unwrap();
            far_future(&mut response);
            Ok(response.to_string().into_bytes())
        });
        let service = service(client);
        let PaymentOptions::Quotes(quotes) = service.payment_options("pay_123", &addresses()).await.unwrap() else {
            panic!("Expected quotes");
        };
        let selected = quotes.quotes.first().unwrap().clone();

        let result = service.prepare_payment(&quotes, &selected, &addresses()).await;

        let paths = requested.lock().unwrap().clone();
        assert!(paths.iter().filter(|path| path.contains("/options")).count() == 2, "expected a requote, got {paths:?}");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prepared_payment_refetches_before_a_dead_quote_is_used() {
        let requested = Arc::new(Mutex::new(Vec::<String>::new()));
        let seen = requested.clone();
        let client = MockClient::new().with_post(move |path: &str, _| {
            seen.lock().unwrap().push(path.to_string());
            let mut response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/options_response.json")).unwrap();
            far_future(&mut response);
            Ok(response.to_string().into_bytes())
        });
        let service = service(client);
        let PaymentOptions::Quotes(quotes) = service.payment_options("pay_123", &addresses()).await.unwrap() else {
            panic!("Expected quotes");
        };
        let expired = PaymentQuote {
            expires_at: Some(Utc::now() - Duration::seconds(1)),
            ..quotes.quotes.first().unwrap().clone()
        };

        let _ = service.prepare_payment(&quotes, &expired, &addresses()).await;

        let paths = requested.lock().unwrap().clone();
        assert_eq!(paths.iter().filter(|path| path.contains("/options")).count(), 2);
        assert_eq!(paths.iter().filter(|path| path.contains("/fetch")).count(), 1);
    }

    #[test]
    fn test_get_quote_keeps_the_chosen_asset() {
        let quote = |chain: Chain| PaymentQuote {
            id: chain.as_ref().to_string(),
            payment_id: "pay_123".to_string(),
            amount: PaymentAmount {
                asset_id: AssetId::from_chain(chain),
                value: "1".to_string(),
                symbol: chain.as_ref().to_string(),
                decimals: 6,
            },
            expires_at: None,
            collect_data_url: None,
            provider_data: "{}".to_string(),
        };
        let quotes = PaymentQuotes {
            merchant: PaymentMerchant {
                name: "Merchant".to_string(),
                icon_url: None,
            },
            price: None,
            expires_at: None,
            quotes: vec![quote(Chain::Ethereum), quote(Chain::Bitcoin)],
        };

        let chosen = WalletConnectPayService::<MockClient>::get_quote(&quotes, &AssetId::from_chain(Chain::Bitcoin)).unwrap();
        assert_eq!(chosen.amount.asset_id, AssetId::from_chain(Chain::Bitcoin));

        let gone = WalletConnectPayService::<MockClient>::get_quote(&quotes, &AssetId::from_chain(Chain::Solana));
        assert_eq!(gone, Err(PaymentError::QuoteExpired));
    }

    fn addresses() -> Vec<ChainAddress> {
        vec![ChainAddress::new(Chain::Ethereum, "0x1".to_string()), ChainAddress::new(Chain::Bitcoin, "bc1".to_string())]
    }

    fn far_future(response: &mut serde_json::Value) {
        response["info"]["expiresAt"] = serde_json::json!(4102444800i64);
    }

    #[tokio::test]
    async fn test_get_payment_options() {
        let service = service_with_response(far_future);
        let prepared = service.payment_options("pay_123", &addresses()).await.unwrap();

        let PaymentOptions::Quotes(quotes) = prepared else {
            panic!("Expected Ready, got {prepared:?}");
        };
        assert_eq!(quotes.merchant.name, "Gem Wallet Test Merchant");
        let quote = quotes.quotes.first().unwrap();
        assert_eq!(quote.payment_id, "pay_123");
        assert_eq!(quote.amount.symbol, "USDC");
        assert_eq!(quote.amount.value, "50000000");
        assert_eq!(WalletConnectPayService::<MockClient>::option(quote).unwrap().id, "opt_1");
    }

    #[tokio::test]
    async fn test_get_payment_options_collect_data() {
        let service = service_with_response(|response| {
            far_future(response);
            response["options"][0]["collectData"] = serde_json::json!({"url": "https://data-collection.walletconnect.com/ic/pay_123"});
        });

        let prepared = service.payment_options("pay_123", &addresses()).await.unwrap();
        let PaymentOptions::Quotes(quotes) = prepared else {
            panic!("Expected Ready, got {prepared:?}");
        };
        assert_eq!(
            quotes.quotes.first().unwrap().collect_data_url.as_deref(),
            Some("https://data-collection.walletconnect.com/ic/pay_123")
        );
    }

    #[tokio::test]
    async fn test_get_payment_options_settled() {
        let service = service_with_response(|response| {
            far_future(response);
            response["info"]["status"] = serde_json::json!("succeeded");
        });

        let options = service.payment_options("pay_123", &addresses()).await.unwrap();
        assert!(matches!(options, PaymentOptions::Outcome(outcome) if outcome.status == PaymentStatus::Succeeded));

        let processing = service_with_response(|response| {
            far_future(response);
            response["info"]["status"] = serde_json::json!("processing");
        });
        let options = processing.payment_options("pay_123", &addresses()).await.unwrap();
        assert!(matches!(options, PaymentOptions::Outcome(outcome) if outcome.status == PaymentStatus::Processing));
    }

    #[tokio::test]
    async fn test_get_payment_options_rejects_unpayable() {
        let failed = service_with_response(|response| {
            far_future(response);
            response["info"]["status"] = serde_json::json!("failed");
        });
        assert_eq!(failed.payment_options("pay_123", &addresses()).await, Err(PaymentError::PaymentExpired));

        let expired = service_with_response(|response| {
            response["info"]["expiresAt"] = serde_json::json!(1);
        });
        assert_eq!(expired.payment_options("pay_123", &addresses()).await, Err(PaymentError::PaymentExpired));

        let no_options = service_with_response(|response| {
            far_future(response);
            response["options"] = serde_json::json!([]);
        });
        assert_eq!(no_options.payment_options("pay_123", &addresses()).await, Err(PaymentError::NoPaymentOptions));

        let unsupported_addresses = vec![ChainAddress::new(Chain::Bitcoin, "bc1".to_string())];
        assert_eq!(
            service_with_response(far_future).payment_options("pay_123", &unsupported_addresses).await,
            Err(PaymentError::UnsupportedAccounts)
        );
    }

    #[test]
    fn test_payment_quotes_offer_options_asking_for_no_personal_data_first() {
        let option = |id: &str, collect_data_url: Option<&str>| QuotedOption {
            id: id.to_string(),
            expires_at: None,
            chain: Chain::Ethereum,
            amount: PaymentAmount {
                asset_id: AssetId::from(Chain::Ethereum, None),
                value: "1".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            collect_data_url: collect_data_url.map(str::to_string),
            provider_data: format!("{{\"id\":\"{id}\"}}"),
        };

        let quotes = WalletConnectPayService::<MockClient>::payment_quotes(
            "pay_123",
            vec![option("opt_form", Some("https://pay.walletconnect.com/collect?pid=pay_123")), option("opt_plain", None)],
        )
        .unwrap();

        assert_eq!(quotes.len(), 2);
        assert!(quotes[0].collect_data_url.is_none());
        assert!(quotes[1].collect_data_url.is_some());
        assert!(quotes.iter().all(|quote| quote.payment_id == "pay_123"));
    }

    #[test]
    fn test_payment_quotes_reject_a_collection_url_off_the_payment_host() {
        let option = |url: &str| QuotedOption {
            id: "opt_form".to_string(),
            expires_at: None,
            chain: Chain::Ethereum,
            amount: PaymentAmount {
                asset_id: AssetId::from(Chain::Ethereum, None),
                value: "1".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            collect_data_url: Some(url.to_string()),
            provider_data: "{}".to_string(),
        };

        for url in [
            "https://evil.com/collect",
            "http://pay.walletconnect.com/collect",
            "https://pay.walletconnect.com.evil.com/collect",
            "https://notwalletconnect.com/collect",
            "not a url",
        ] {
            assert!(
                WalletConnectPayService::<MockClient>::payment_quotes("pay_123", vec![option(url)]).is_err(),
                "{url} was accepted"
            );
        }

        assert!(WalletConnectPayService::<MockClient>::payment_quotes("pay_123", vec![option("https://pay.walletconnect.com/collect")]).is_ok());
        assert!(WalletConnectPayService::<MockClient>::payment_quotes("pay_123", vec![option("https://data-collection.walletconnect.com/ic/pay_123")]).is_ok());
    }
}
