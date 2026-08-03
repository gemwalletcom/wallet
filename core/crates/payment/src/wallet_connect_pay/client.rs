use gem_client::{Client, build_path_with_query};
use std::collections::HashMap;

use crate::wallet_connect_pay::error::WalletConnectPayError;
use crate::wallet_connect_pay::model::{
    ConfirmPaymentRequest, FetchActionsRequest, FetchActionsResponse, PaymentOptionsRequest, PaymentOptionsResponse, PaymentStatusResponse, WalletConnectPayAction,
    WalletConnectPayActionResult,
};
use primitives::payment_decoder::wallet_connect_pay::is_payment_id;

pub const WALLET_CONNECT_PAY_API_URL: &str = "https://api.pay.walletconnect.com";
const WALLET_CONNECT_PAY_VERSION: &str = "2026-02-18";

const HEADER_WALLET_CONNECT_PAY_VERSION: &str = "WCP-Version";
const HEADER_APP_ID: &str = "App-Id";
const HEADER_CLIENT_ID: &str = "Client-Id";
const HEADER_SDK_NAME: &str = "Sdk-Name";
const HEADER_SDK_VERSION: &str = "Sdk-Version";

const SDK_NAME: &str = "gem-wallet";
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

const QUERY_INCLUDE_PAYMENT_INFO: &str = "includePaymentInfo";
const QUERY_MAX_POLL_MS: &str = "maxPollMs";
const MAX_POLL_MS: i64 = 60_000;

#[derive(Debug, Clone)]
pub struct WalletConnectPayAuth {
    pub app_id: String,
    pub client_id: String,
}

impl WalletConnectPayAuth {
    pub fn new(app_id: String, client_id: String) -> Self {
        Self { app_id, client_id }
    }
}

#[derive(Debug)]
pub struct WalletConnectPayClient<C: Client> {
    client: C,
    auth: WalletConnectPayAuth,
}

impl<C: Client> WalletConnectPayClient<C> {
    pub fn new(client: C, auth: WalletConnectPayAuth) -> Self {
        Self { client, auth }
    }

    pub async fn get_options(&self, payment_id: &str, accounts: Vec<String>) -> Result<PaymentOptionsResponse, WalletConnectPayError> {
        let path = Self::path(payment_id, "/options", &[(QUERY_INCLUDE_PAYMENT_INFO, "true".to_string())])?;
        let request = PaymentOptionsRequest { accounts };
        Ok(self.client.post_with(&path, &request, self.headers()).await?)
    }

    pub async fn get_actions(&self, payment_id: &str, option_id: &str, data: String) -> Result<Vec<WalletConnectPayAction>, WalletConnectPayError> {
        let path = Self::path(payment_id, "/fetch", &[])?;
        let request = FetchActionsRequest {
            option_id: option_id.to_string(),
            data,
        };
        let response: FetchActionsResponse = self.client.post_with(&path, &request, self.headers()).await?;
        Ok(response.actions)
    }

    pub async fn confirm(&self, payment_id: &str, option_id: &str, action_results: Vec<String>) -> Result<PaymentStatusResponse, WalletConnectPayError> {
        let path = Self::path(payment_id, "/confirm", &[(QUERY_MAX_POLL_MS, MAX_POLL_MS.to_string())])?;
        let request = ConfirmPaymentRequest {
            option_id: option_id.to_string(),
            results: action_results.into_iter().map(WalletConnectPayActionResult::wallet_rpc).collect(),
        };
        Ok(self.client.post_with(&path, &request, self.headers()).await?)
    }

    pub async fn cancel(&self, payment_id: &str) -> Result<(), WalletConnectPayError> {
        let path = Self::path(payment_id, "/cancel", &[])?;
        let _: serde_json::Value = self.client.post_with(&path, &serde_json::Value::Null, self.headers()).await?;
        Ok(())
    }

    pub async fn get_status(&self, payment_id: &str) -> Result<PaymentStatusResponse, WalletConnectPayError> {
        let path = Self::path(payment_id, "/status", &[(QUERY_MAX_POLL_MS, MAX_POLL_MS.to_string())])?;
        Ok(self.client.get_with(&path, &[], self.headers()).await?)
    }

    fn path(payment_id: &str, suffix: &str, query: &[(&str, String)]) -> Result<String, WalletConnectPayError> {
        if !is_payment_id(payment_id) {
            return Err(WalletConnectPayError::InvalidRequest(format!("Invalid payment id: {payment_id}")));
        }
        let path = format!("/v1/gateway/payment/{payment_id}{suffix}");
        if query.is_empty() {
            return Ok(path);
        }
        build_path_with_query(&path, &query).map_err(|error| WalletConnectPayError::InvalidRequest(error.to_string()))
    }

    fn headers(&self) -> HashMap<String, String> {
        [
            (HEADER_WALLET_CONNECT_PAY_VERSION.to_string(), WALLET_CONNECT_PAY_VERSION.to_string()),
            (HEADER_SDK_NAME.to_string(), SDK_NAME.to_string()),
            (HEADER_SDK_VERSION.to_string(), SDK_VERSION.to_string()),
            (HEADER_APP_ID.to_string(), self.auth.app_id.clone()),
            (HEADER_CLIENT_ID.to_string(), self.auth.client_id.clone()),
        ]
        .into_iter()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::WalletRpcAction;
    use crate::wallet_connect_pay::testkit::{TEST_APP_ID, TEST_CLIENT_ID};
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::PaymentStatus;

    fn client(mock: MockClient) -> WalletConnectPayClient<MockClient> {
        WalletConnectPayClient::new(mock, WalletConnectPayAuth::mock())
    }

    #[tokio::test]
    async fn test_get_options() {
        let mock = MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(headers.get(HEADER_WALLET_CONNECT_PAY_VERSION).unwrap(), WALLET_CONNECT_PAY_VERSION);
            assert_eq!(headers.get(HEADER_SDK_NAME).unwrap(), SDK_NAME);
            assert_eq!(headers.get(HEADER_APP_ID).unwrap(), TEST_APP_ID);
            assert_eq!(headers.get(HEADER_CLIENT_ID).unwrap(), TEST_CLIENT_ID);
            assert!(path.ends_with("/v1/gateway/payment/pay_123/options?includePaymentInfo=true"));
            assert_eq!(serde_json::from_slice::<serde_json::Value>(body).unwrap()["accounts"], serde_json::json!(["eip155:1:0x1"]));
            Ok(include_str!("../../testdata/options_response.json").as_bytes().to_vec())
        });

        let response = client(mock).get_options("pay_123", vec!["eip155:1:0x1".to_string()]).await.unwrap();
        let info = response.info.unwrap();
        assert_eq!(info.status, PaymentStatus::RequiresAction);
        assert_eq!(info.merchant.name, "Gem Wallet Test Merchant");
        assert_eq!(info.amount.value, "5000");
        assert_eq!(info.amount.display.decimals, 2);

        let option = response.options.unwrap().remove(0);
        assert_eq!(option.id, "opt_1");
        assert_eq!(option.account, "eip155:1:0x1085c5f70F7F7591D97da281A64688385455c2bD");
        assert_eq!(option.amount.display.asset_symbol, "USDC");
        assert!(option.actions.is_empty());
        assert!(option.collect_data.is_none());
    }

    #[tokio::test]
    async fn test_fetch_actions() {
        let mock = MockClient::new().with_post(|path, body| {
            assert!(path.ends_with("/v1/gateway/payment/pay_123/fetch"));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(body).unwrap(),
                serde_json::json!({"optionId": "opt_1", "data": ""})
            );
            Ok(include_str!("../../testdata/fetch_response_transfer_authorization.json").as_bytes().to_vec())
        });

        let actions = client(mock).get_actions("pay_123", "opt_1", String::new()).await.unwrap();
        assert!(matches!(actions.as_slice(), [WalletConnectPayAction::WalletRpc(WalletRpcAction { method, .. })] if method == "eth_signTypedData_v4"));
    }

    #[tokio::test]
    async fn test_confirm() {
        let mock = MockClient::new().with_post(|path, body| {
            assert!(path.ends_with("/v1/gateway/payment/pay_123/confirm?maxPollMs=60000"));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(body).unwrap(),
                serde_json::json!({
                    "optionId": "opt_1",
                    "results": [{ "type": "walletRpc", "data": ["0xsignature"] }]
                })
            );
            Ok(br#"{"status":"processing","isFinal":false,"pollInMs":1000}"#.to_vec())
        });

        let response = client(mock).confirm("pay_123", "opt_1", vec!["0xsignature".to_string()]).await.unwrap();
        assert_eq!(response.status, PaymentStatus::Processing);
        assert!(!response.is_final);
        assert_eq!(response.poll_in_ms, Some(1000));
    }

    #[tokio::test]
    async fn test_get_status() {
        let mock = MockClient::new().with_get(|path| {
            assert!(path.ends_with("/v1/gateway/payment/pay_123/status?maxPollMs=60000"));
            Ok(include_str!("../../testdata/status_response_succeeded.json").as_bytes().to_vec())
        });

        let response = client(mock).get_status("pay_123").await.unwrap();
        assert_eq!(response.status, PaymentStatus::Succeeded);
        assert!(response.is_final);
        assert_eq!(response.info.unwrap().tx_id, "test:pay_b9a2ecc101KYJAYCGQZ9E0K6NY7SR7YVV4");
    }

    #[test]
    fn test_path_rejects_unsafe_payment_id() {
        assert!(WalletConnectPayClient::<MockClient>::path("pay_123", "/status", &[]).is_ok());
        assert!(WalletConnectPayClient::<MockClient>::path("pay_123/cancel", "/status", &[]).is_err());
        assert!(WalletConnectPayClient::<MockClient>::path("pay_123?x=1", "", &[]).is_err());
        assert!(WalletConnectPayClient::<MockClient>::path("pay 123", "", &[]).is_err());
        assert!(WalletConnectPayClient::<MockClient>::path("", "/status", &[]).is_err());
    }

    #[tokio::test]
    async fn test_error_mapping() {
        let mock = MockClient::new().with_get(|_| Err(ClientError::Http { status: 410, body: vec![] }));
        assert_eq!(client(mock).get_status("pay_123").await, Err(WalletConnectPayError::PaymentExpired));

        assert_eq!(
            WalletConnectPayError::from(ClientError::Http { status: 404, body: vec![] }),
            WalletConnectPayError::PaymentNotFound
        );
        assert_eq!(
            WalletConnectPayError::from(ClientError::Http { status: 409, body: vec![] }),
            WalletConnectPayError::QuoteExpired
        );
        assert_eq!(
            WalletConnectPayError::from(ClientError::Http { status: 429, body: vec![] }),
            WalletConnectPayError::RateLimited
        );
        assert!(matches!(
            WalletConnectPayError::from(ClientError::Http {
                status: 400,
                body: b"bad".to_vec()
            }),
            WalletConnectPayError::InvalidRequest(_)
        ));
        assert!(matches!(
            WalletConnectPayError::from(ClientError::Http { status: 500, body: vec![] }),
            WalletConnectPayError::Network(_)
        ));
    }
}
