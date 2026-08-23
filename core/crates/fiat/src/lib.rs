pub mod client;
pub mod error;
pub mod fiat_cacher_client;
pub mod hmac_signature;
pub mod ip_check_client;
pub mod model;
pub mod provider;
pub mod providers;
pub mod rsa_signature;
pub mod transaction_info_mapper;
pub mod webhook;

pub use provider::FiatProvider;
pub use webhook::FiatWebhookRequest;

use crate::providers::{BanxaClient, FlashnetClient, MercuryoClient, MoonPayClient, PaybisClient, TransakClient};
use gem_client::ReqwestClient;
use settings::Settings;
use std::time::Duration;

pub use client::FiatClient;

fn request_client(timeout: Duration) -> reqwest::Client {
    gem_client::builder().timeout(timeout).build().expect("fiat HTTP client configuration is valid")
}
pub use fiat_cacher_client::{CachedFiatQuoteData, FiatCacherClient};
pub use ip_check_client::{IPAddressInfo, IPCheckClient};
pub use transaction_info_mapper::fiat_transaction_info;

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub mod testkit;

pub struct FiatProviderFactory {}
impl FiatProviderFactory {
    pub fn new_providers(settings: Settings) -> Vec<Box<dyn FiatProvider + Send + Sync>> {
        let request_client = request_client(settings.fiat.timeout);

        let moonpay = MoonPayClient::new(
            ReqwestClient::new(settings.fiat.moonpay.url.clone(), request_client.clone()),
            settings.fiat.moonpay.key.public.clone(),
            settings.fiat.moonpay.key.secret.clone(),
            settings.fiat.moonpay.webhook.key.secret.clone(),
        );
        let mercuryo = MercuryoClient::new(
            ReqwestClient::new(settings.fiat.mercuryo.url.clone(), request_client.clone()),
            settings.fiat.mercuryo.key.public.clone(),
            settings.fiat.mercuryo.key.secret.clone(),
            settings.fiat.mercuryo.webhook.key.secret.clone(),
        );
        let transak = TransakClient::new(
            ReqwestClient::new(settings.fiat.transak.url, request_client.clone()),
            ReqwestClient::new(settings.fiat.transak.gateway.url, request_client.clone()),
            settings.fiat.transak.key.public,
            settings.fiat.transak.key.secret,
            settings.fiat.transak.referrer.domain,
        );
        let banxa = BanxaClient::new(
            ReqwestClient::new(settings.fiat.banxa.api.url, request_client.clone()),
            settings.fiat.banxa.redirect.url,
            settings.fiat.banxa.partner,
            settings.fiat.banxa.key.secret,
            settings.fiat.banxa.webhook.key.secret,
        );
        let paybis = PaybisClient::new(
            ReqwestClient::new(settings.fiat.paybis.url, request_client.clone()),
            settings.fiat.paybis.key.public,
            settings.fiat.paybis.key.secret,
        );
        let flashnet = FlashnetClient::new(
            ReqwestClient::new(settings.fiat.flashnet.url, request_client.clone()),
            settings.fiat.flashnet.key.secret,
            settings.fiat.flashnet.key.public,
            settings.fiat.flashnet.webhook.key.secret,
        );

        vec![
            Box::new(moonpay),
            Box::new(mercuryo),
            Box::new(transak),
            Box::new(banxa),
            Box::new(paybis),
            Box::new(flashnet),
        ]
    }

    pub fn new_ip_check_client(settings: Settings) -> IPCheckClient {
        let request_client = request_client(settings.fiat.timeout);
        let moonpay = MoonPayClient::new(
            ReqwestClient::new(settings.fiat.moonpay.url.clone(), request_client),
            settings.fiat.moonpay.key.public.clone(),
            settings.fiat.moonpay.key.secret.clone(),
            settings.fiat.moonpay.webhook.key.secret.clone(),
        );
        IPCheckClient::new(moonpay)
    }
}
