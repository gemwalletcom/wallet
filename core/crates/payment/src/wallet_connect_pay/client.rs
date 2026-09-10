use gem_client::{Client, ClientExt};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

use crate::error::PaymentError;
use crate::wallet_connect_pay::config::WalletConnectPayAuth;
use crate::wallet_connect_pay::model::{
    ConfirmPaymentRequest, FetchActionsRequest, FetchActionsResponse, PaymentActions, PaymentOptionsRequest, PaymentOptionsResponse, PaymentStatusResponse,
    WalletConnectPayActionResult,
};
use crate::wallet_connect_pay::target::WalletConnectPayTarget;

pub(super) const WALLET_CONNECT_PAY_API_URL: &str = "https://api.pay.walletconnect.com";
const WALLET_CONNECT_PAY_VERSION: &str = "2026-02-18";

const HEADER_WALLET_CONNECT_PAY_VERSION: &str = "WCP-Version";
const HEADER_APP_ID: &str = "App-Id";
const HEADER_CLIENT_ID: &str = "Client-Id";

const COLLECT_DATA_REQUIRED_MESSAGE: &str = "IC data required";

#[derive(Debug)]
pub(super) struct WalletConnectPayClient<C: Client> {
    client: C,
    headers: HashMap<String, String>,
}

impl<C: Client> WalletConnectPayClient<C> {
    pub(super) fn new(client: C, auth: WalletConnectPayAuth) -> Self {
        let headers = HashMap::from([
            (HEADER_WALLET_CONNECT_PAY_VERSION.to_string(), WALLET_CONNECT_PAY_VERSION.to_string()),
            (HEADER_APP_ID.to_string(), auth.app_id),
            (HEADER_CLIENT_ID.to_string(), auth.client_id),
        ]);
        Self { client, headers }
    }

    pub(super) async fn get_options(&self, payment_id: &str, accounts: &[String]) -> Result<PaymentOptionsResponse, PaymentError> {
        let target = WalletConnectPayTarget::Options {
            payment_id: payment_id.to_string(),
        };
        self.post(target, &PaymentOptionsRequest { accounts }).await
    }

    pub(super) async fn get_actions(&self, payment_id: &str, option_id: &str, data: String) -> Result<PaymentActions, PaymentError> {
        let target = WalletConnectPayTarget::Fetch {
            payment_id: payment_id.to_string(),
        };
        let request = FetchActionsRequest {
            option_id: option_id.to_string(),
            data,
        };
        match self.post(target, &request).await {
            Ok(FetchActionsResponse { actions }) => Ok(PaymentActions::Ready(actions)),
            Err(PaymentError::InvalidRequest { reason }) if reason.contains(COLLECT_DATA_REQUIRED_MESSAGE) => Ok(PaymentActions::CollectData),
            Err(error) => Err(error),
        }
    }

    pub(super) async fn confirm(&self, payment_id: &str, option_id: &str, transaction_hash: String) -> Result<PaymentStatusResponse, PaymentError> {
        let target = WalletConnectPayTarget::Confirm {
            payment_id: payment_id.to_string(),
        };
        let request = ConfirmPaymentRequest {
            option_id: option_id.to_string(),
            results: vec![WalletConnectPayActionResult::wallet_rpc(transaction_hash)],
        };
        self.post(target, &request).await
    }

    async fn post<T: DeserializeOwned + Send, R: Serialize + Send + Sync>(&self, target: WalletConnectPayTarget, request: &R) -> Result<T, PaymentError> {
        self.client.post(target, request).headers(self.headers.clone()).await.map_err(PaymentError::from)
    }
}
