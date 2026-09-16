use std::collections::HashMap;

use gem_client::{ClientError, ClientExt, ReqwestClient, Target};
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::TransactionId;
use serde_json::Value;
use settings_chain::BroadcastProviders;

use crate::config::WebhookConfig;
use crate::proxy::proxy_request::ProxyRequest;

#[derive(Debug, Clone)]
pub struct DynodeBroadcastWebhookClient {
    enabled: bool,
    url: String,
    token: String,
    client: reqwest::Client,
}

impl DynodeBroadcastWebhookClient {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            enabled: config.enabled,
            url: config.url,
            token: config.token,
            client: gem_client::reqwest_client(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            token: String::new(),
            client: gem_client::reqwest_client(),
        }
    }

    pub fn notify_broadcast(&self, request: &ProxyRequest, response_status: u16, response_body: &[u8], broadcast_providers: &BroadcastProviders) {
        if !self.should_notify(request, response_status, broadcast_providers) {
            return;
        }

        let Some(payload) = self.extract_payload(request, response_body, broadcast_providers) else {
            return;
        };

        self.spawn_notify(payload, request.id.clone());
    }

    fn should_notify(&self, request: &ProxyRequest, response_status: u16, broadcast_providers: &BroadcastProviders) -> bool {
        self.enabled && !self.url.is_empty() && !self.token.is_empty() && request.is_broadcast(broadcast_providers) && is_success_status(response_status)
    }

    fn extract_payload(&self, request: &ProxyRequest, response_body: &[u8], broadcast_providers: &BroadcastProviders) -> Option<TransactionId> {
        let identifier = broadcast_providers.decode_transaction_broadcast(request.chain, &request.body, response_body).ok()?;
        Some(TransactionId::new(request.chain, identifier))
    }

    fn spawn_notify(&self, payload: TransactionId, request_id: String) {
        let url = self.url.clone();
        let token = self.token.clone();
        let client = self.client.clone();

        tokio::spawn(Self::deliver(client, url, token, payload, request_id));
    }

    async fn deliver(client: reqwest::Client, url: String, token: String, payload: TransactionId, request_id: String) {
        let transaction_id = payload.to_string();
        let client = ReqwestClient::new(url, client);
        let headers = HashMap::from([("Authorization".to_string(), format!("Bearer {token}"))]);

        match client.post::<_, Value>(WebhookTarget::Broadcast, &payload).headers(headers).await {
            Ok(_) => {
                info_with_fields!("broadcast webhook delivered", transaction_id = transaction_id.as_str(), request_id = request_id.as_str(),);
            }
            Err(ClientError::Http { status, .. }) => {
                info_with_fields!(
                    "broadcast webhook delivery failed",
                    transaction_id = transaction_id.as_str(),
                    request_id = request_id.as_str(),
                    status = status,
                );
            }
            Err(err) => {
                error_with_fields!(
                    "broadcast webhook request failed",
                    &err,
                    transaction_id = transaction_id.as_str(),
                    request_id = request_id.as_str(),
                );
            }
        }
    }
}

fn is_success_status(status: u16) -> bool {
    (200..300).contains(&status)
}

#[derive(Clone, Debug)]
enum WebhookTarget {
    Broadcast,
}

impl Target for WebhookTarget {
    fn path(&self) -> String {
        match self {
            Self::Broadcast => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use reqwest::Method;

    use super::*;

    #[test]
    fn test_extract_payload() {
        let client = DynodeBroadcastWebhookClient::disabled();
        let providers = BroadcastProviders::from_chains([Chain::HyperCore]);
        let request = ProxyRequest::mock(Chain::HyperCore, Method::POST, "/exchange", br#"{"action":{"type":"updateLeverage"},"nonce":123}"#);
        let response = br#"{"status":"ok","response":{"type":"default"}}"#;
        assert_eq!(
            client.extract_payload(&request, response, &providers),
            Some(TransactionId::new(Chain::HyperCore, "action:123".into()))
        );
    }
}
