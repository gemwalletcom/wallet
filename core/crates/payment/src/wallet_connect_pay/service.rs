use gem_client::Client;
use primitives::{AssetId, ChainAddress, PaymentOptions, PaymentOutcome, PaymentQuote, PaymentQuotes};

use crate::error::PaymentError;
use crate::wallet_connect_pay::account::{get_account_identifier, is_supported};
use crate::wallet_connect_pay::action_mapper::map_wallet_rpc;
use crate::wallet_connect_pay::client::{WalletConnectPayAuth, WalletConnectPayClient};
use crate::wallet_connect_pay::model::{PaymentOption, WalletConnectPayAction, WalletRpcAction};
use crate::wallet_connect_pay::payment_mapper;
use primitives::{PaymentAction, PaymentQuoteData};

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

    pub async fn get_quote_data(&self, payment_id: &str, quote: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PaymentQuoteData, PaymentError> {
        match self.action(payment_id, quote).await {
            Ok(action) => Ok(PaymentQuoteData { quote: quote.clone(), action }),
            Err(PaymentError::QuoteExpired) => self.requote(payment_id, quote, addresses).await,
            Err(error) => Err(error),
        }
    }

    async fn requote(&self, payment_id: &str, expired: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PaymentQuoteData, PaymentError> {
        let quotes = match self.get_options(payment_id, addresses).await? {
            PaymentOptions::Quotes(quotes) => quotes,
            PaymentOptions::Outcome(_) => return Err(PaymentError::PaymentExpired),
        };
        let quote = Self::get_quote(&quotes, &expired.asset_id)?;
        let action = self.action(payment_id, &quote).await?;
        Ok(PaymentQuoteData { quote, action })
    }

    pub async fn confirm(&self, payment_id: &str, quote_id: &str, transaction_hash: String) -> Result<PaymentOutcome, PaymentError> {
        let response = self.client.confirm(payment_id, quote_id, transaction_hash).await?;
        Ok(payment_mapper::map_payment_outcome(response))
    }

    pub async fn get_options(&self, payment_id: &str, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        let identifiers: Vec<String> = addresses
            .iter()
            .filter(|address| is_supported(address.chain))
            .filter_map(|address| get_account_identifier(address.chain, &address.address))
            .collect();
        if identifiers.is_empty() {
            return Err(PaymentError::NoPaymentOptions);
        }
        let response = self.client.get_options(payment_id, &identifiers).await?;
        payment_mapper::map_options(response, payment_id, &identifiers)
    }

    fn get_quote(quotes: &PaymentQuotes, asset_id: &AssetId) -> Result<PaymentQuote, PaymentError> {
        quotes.quotes.iter().find(|quote| &quote.asset_id == asset_id).cloned().ok_or(PaymentError::QuoteExpired)
    }

    async fn action(&self, payment_id: &str, quote: &PaymentQuote) -> Result<PaymentAction, PaymentError> {
        let option = Self::option(quote)?;
        match self.wallet_rpc_actions(payment_id, &option).await?.as_slice() {
            [action] => map_wallet_rpc(&option.account, &quote.value, action),
            actions => Err(PaymentError::InvalidRequest(format!("Payment asks for {} actions", actions.len()))),
        }
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

    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use primitives::{AssetId, Chain, PaymentLink, PaymentMerchant};
    use std::sync::{Arc, Mutex};

    fn service(mock: MockClient) -> WalletConnectPayService<MockClient> {
        WalletConnectPayService::new(mock, WalletConnectPayAuth::mock())
    }

    fn service_with_response() -> WalletConnectPayService<MockClient> {
        service(MockClient::new().with_post(move |_, _| {
            Ok(include_str!("../../testdata/options_response_coin.json").as_bytes().to_vec())
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
            Ok(include_str!("../../testdata/options_response_coin.json").as_bytes().to_vec())
        });
        let service = service(client);
        let PaymentOptions::Quotes(quotes) = service.get_options("pay_123", &addresses()).await.unwrap() else {
            panic!("Expected quotes");
        };
        let selected = quotes.quotes.first().unwrap().clone();

        let result = service.get_quote_data("pay_123", &selected, &addresses()).await;

        let paths = requested.lock().unwrap().clone();
        assert!(paths.iter().filter(|path| path.contains("/options")).count() == 2, "expected a requote, got {paths:?}");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_quote_keeps_the_chosen_asset() {
        let quote = |chain: Chain| PaymentQuote {
            id: chain.as_ref().to_string(),
            link: PaymentLink::WalletConnectPay("pay_123".to_string()),
            asset_id: AssetId::from_chain(chain),
            value: 1u32.into(),
            collect_data_url: None,
            provider_data: "{}".to_string(),
        };
        let quotes = PaymentQuotes {
            merchant: PaymentMerchant {
                name: "Merchant".to_string(),
                icon_url: None,
            },
            price: None,
            quotes: vec![quote(Chain::Ethereum), quote(Chain::Bitcoin)],
        };

        let chosen = WalletConnectPayService::<MockClient>::get_quote(&quotes, &AssetId::from_chain(Chain::Bitcoin)).unwrap();
        assert_eq!(chosen.asset_id, AssetId::from_chain(Chain::Bitcoin));

        let gone = WalletConnectPayService::<MockClient>::get_quote(&quotes, &AssetId::from_chain(Chain::Solana));
        assert_eq!(gone, Err(PaymentError::QuoteExpired));
    }

    fn addresses() -> Vec<ChainAddress> {
        vec![
            ChainAddress::new(Chain::Ethereum, "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string()),
            ChainAddress::new(Chain::Bitcoin, "bc1".to_string()),
        ]
    }

    #[tokio::test]
    async fn test_get_quote_data_signs_the_gateway_settlement_call() {
        let client = MockClient::new().with_post(|path: &str, _| {
            let fixture = if path.contains("/fetch") {
                include_str!("../../testdata/fetch_response_native.json").to_string()
            } else {
                let mut response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/options_response_native.json")).unwrap();
                response["info"]["expiresAt"] = serde_json::json!(4102444800i64);
                response["options"][1]["expiresAt"] = serde_json::json!(4102444800i64);
                response.to_string()
            };
            Ok(fixture.into_bytes())
        });
        let service = service(client);
        let addresses = vec![ChainAddress::new(Chain::Ethereum, "0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB".to_string())];
        let PaymentOptions::Quotes(quotes) = service.get_options("pay_123", &addresses).await.unwrap() else {
            panic!("Expected quotes");
        };

        assert_eq!(quotes.quotes.len(), 1);
        let quote = quotes.quotes.first().unwrap();
        assert_eq!(quote.value, 14192816625800u64.into());

        let payment = service.get_quote_data("pay_123", quote, &addresses).await.unwrap();

        assert_eq!(
            payment.action,
            PaymentAction::Send {
                chain: Chain::Ethereum,
                recipient: "0x57b2b4288220005234c0e88a04a7943193971d21".to_string(),
                value: 14192816625800u64.into(),
                data: "0xd390648800000000000000000000000057b2b4288220005234c0e88a04a7943193971d2100000000000000000000000092abce21234d71ec443e679f3a1feafd3fc830fb000000000000000000000000505f60096888ea3264dc8778bb9bf1809daa042c000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000000000000000000000000000000000000000000000000000ce885cb1888000000000000000000000000000000000000000000000000000000006a860ea2".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_options() {
        let service = service_with_response();
        let prepared = service.get_options("pay_123", &addresses()).await.unwrap();

        let PaymentOptions::Quotes(quotes) = prepared else {
            panic!("Expected Ready, got {prepared:?}");
        };
        assert_eq!(quotes.merchant.name, "Gem Wallet Test Merchant");
        let quote = quotes.quotes.first().unwrap();
        assert_eq!(quote.link, PaymentLink::WalletConnectPay("pay_123".to_string()));
        assert_eq!(quote.value, 16975688363325440u64.into());
        assert_eq!(WalletConnectPayService::<MockClient>::option(quote).unwrap().id, "opt_eth");
    }

    #[tokio::test]
    async fn test_options_refuses_a_wallet_with_no_account_the_gateway_accepts() {
        let unsupported = vec![ChainAddress::new(Chain::Bitcoin, "bc1".to_string())];

        assert_eq!(
            service_with_response().get_options("pay_123", &unsupported).await,
            Err(PaymentError::NoPaymentOptions)
        );
    }
}
