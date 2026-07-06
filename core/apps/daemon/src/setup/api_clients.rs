use primitives::{FiatProviderName, WebhookKind};
use storage::ApiClientScope;
use storage::models::{ApiClientGrant, ApiClientResource};

pub const SETUP_DEV_API_CLIENT_NAME: &str = "test";
pub const SETUP_DEV_API_CLIENT_SECRET: &str = "00000000-0000-0000-0000-000000000000";

pub fn setup_api_client_grants() -> Vec<ApiClientGrant> {
    api_client_access_grants("admin")
        .into_iter()
        .chain(webhook_senders().into_iter().map(|(kind, sender)| ApiClientGrant {
            client_name: format!("webhook:{}:{}", kind.as_ref(), sender),
            scope: ApiClientScope::webhook(kind),
            resource: ApiClientResource::WebhookSender(sender),
        }))
        .collect()
}

pub fn api_client_access_grants(client_name: &str) -> Vec<ApiClientGrant> {
    let globals = [
        ApiClientScope::AdminWrite,
        ApiClientScope::ChainRead,
        ApiClientScope::DevicesRead,
        ApiClientScope::DevicesSubscriptionsRead,
        ApiClientScope::DevicesTransactionsRead,
        ApiClientScope::FiatQuotesRead,
    ]
    .into_iter()
    .map(|scope| ApiClientGrant {
        client_name: client_name.to_string(),
        scope,
        resource: ApiClientResource::Global,
    });

    let webhooks = webhook_senders().into_iter().map(|(kind, sender)| ApiClientGrant {
        client_name: client_name.to_string(),
        scope: ApiClientScope::webhook(kind),
        resource: ApiClientResource::WebhookSender(sender),
    });

    globals.chain(webhooks).collect()
}

fn webhook_senders() -> Vec<(WebhookKind, String)> {
    [(WebhookKind::Transactions, "dynode".to_string()), (WebhookKind::Support, "chatwoot".to_string())]
        .into_iter()
        .chain(FiatProviderName::all().into_iter().map(|provider| (WebhookKind::Fiat, provider.id().to_string())))
        .collect()
}
