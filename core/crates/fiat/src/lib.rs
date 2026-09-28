pub mod error;
pub mod hmac_signature;
pub mod ip_check_client;
pub mod model;
pub mod provider;
pub mod providers;
pub mod quotes;
pub mod rsa_signature;
pub mod transaction_info_mapper;
pub mod webhook;

pub use provider::FiatProvider;
pub use webhook::FiatWebhookRequest;

use crate::providers::{BanxaClient, FlashnetClient, MercuryoClient, MoonPayClient, PaybisClient, TransakClient};
use gem_client::ReqwestClient;
use primitives::AccessTokenCacher;
use settings::Settings;
use std::sync::Arc;
use std::time::Duration;

pub use model::FiatDeviceContext;

fn request_client(timeout: Duration) -> reqwest::Client {
    gem_client::builder().timeout(timeout).build().expect("fiat HTTP client configuration is valid")
}
pub use ip_check_client::{IPAddressInfo, IPCheckClient};
pub use transaction_info_mapper::fiat_transaction_info;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub struct FiatProviderFactory {}
impl FiatProviderFactory {
    pub fn new_providers(settings: &Settings, access_token_cacher: Arc<dyn AccessTokenCacher>) -> Vec<Box<dyn FiatProvider + Send + Sync>> {
        let request_client = request_client(settings.fiat.timeout);

        let moonpay = moonpay_client(settings, request_client.clone());
        let mercuryo = MercuryoClient::new(
            ReqwestClient::new(settings.fiat.mercuryo.url.clone(), request_client.clone()),
            settings.fiat.mercuryo.key.public.clone(),
            settings.fiat.mercuryo.key.secret.clone(),
            settings.fiat.mercuryo.webhook.key.secret.clone(),
        );
        let transak = TransakClient::new(
            ReqwestClient::new(settings.fiat.transak.url.clone(), request_client.clone()),
            ReqwestClient::new(settings.fiat.transak.gateway.url.clone(), request_client.clone()),
            settings.fiat.transak.key.public.clone(),
            settings.fiat.transak.key.secret.clone(),
            settings.fiat.transak.referrer.domain.clone(),
            access_token_cacher,
        );
        let banxa = BanxaClient::new(
            ReqwestClient::new(settings.fiat.banxa.api.url.clone(), request_client.clone()),
            settings.fiat.banxa.redirect.url.clone(),
            settings.fiat.banxa.partner.clone(),
            settings.fiat.banxa.key.secret.clone(),
            settings.fiat.banxa.webhook.key.secret.clone(),
        );
        let paybis = PaybisClient::new(
            ReqwestClient::new(settings.fiat.paybis.url.clone(), request_client.clone()),
            settings.fiat.paybis.key.public.clone(),
            settings.fiat.paybis.key.secret.clone(),
        );
        let flashnet = FlashnetClient::new(
            ReqwestClient::new(settings.fiat.flashnet.url.clone(), request_client),
            settings.fiat.flashnet.key.secret.clone(),
            settings.fiat.flashnet.key.public.clone(),
            settings.fiat.flashnet.webhook.key.secret.clone(),
        );

        vec![Box::new(moonpay), Box::new(mercuryo), Box::new(transak), Box::new(banxa), Box::new(paybis), Box::new(flashnet)]
    }

    pub fn new_ip_check_client(settings: &Settings) -> IPCheckClient {
        IPCheckClient::new(moonpay_client(settings, request_client(settings.fiat.timeout)))
    }
}

fn moonpay_client(settings: &Settings, request_client: reqwest::Client) -> MoonPayClient {
    MoonPayClient::new(
        ReqwestClient::new(settings.fiat.moonpay.url.clone(), request_client),
        settings.fiat.moonpay.key.public.clone(),
        settings.fiat.moonpay.key.secret.clone(),
        settings.fiat.moonpay.webhook.key.secret.clone(),
    )
}
