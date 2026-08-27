use crate::model::FiatDeviceContext;
use primitives::WalletType;

impl FiatDeviceContext {
    pub fn mock() -> Self {
        Self::mock_with_wallet_type(WalletType::Multicoin)
    }

    pub fn mock_with_wallet_type(wallet_type: WalletType) -> Self {
        Self::new(1, 2, wallet_type, "192.0.2.1".to_string())
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
use crate::model::FiatMapping;
#[cfg(all(test, feature = "fiat_integration_tests"))]
use crate::providers::{
    banxa::client::BanxaClient, mercuryo::client::MercuryoClient, moonpay::client::MoonPayClient, paybis::client::PaybisClient, transak::client::TransakClient,
};
#[cfg(all(test, feature = "fiat_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "fiat_integration_tests"))]
use settings::Settings;

#[cfg(all(test, feature = "fiat_integration_tests"))]
fn get_test_settings() -> Settings {
    let settings_path = std::env::current_dir().expect("Failed to get current directory").join("../../Settings.yaml");
    Settings::new_setting_path(settings_path).expect("Failed to load settings for tests")
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_transak_test_client() -> TransakClient {
    let settings = get_test_settings();
    let client = crate::request_client(settings.fiat.timeout);
    TransakClient::new(
        ReqwestClient::new(settings.fiat.transak.url, client.clone()),
        ReqwestClient::new(settings.fiat.transak.gateway.url, client),
        settings.fiat.transak.key.public,
        settings.fiat.transak.key.secret,
        settings.fiat.transak.referrer.domain,
    )
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_moonpay_test_client() -> MoonPayClient {
    let settings = get_test_settings();
    let client = crate::request_client(settings.fiat.timeout);
    MoonPayClient::new(
        ReqwestClient::new(settings.fiat.moonpay.url, client),
        settings.fiat.moonpay.key.public,
        settings.fiat.moonpay.key.secret,
        settings.fiat.moonpay.webhook.key.secret,
    )
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_paybis_test_client() -> PaybisClient {
    let settings = get_test_settings();
    let client = crate::request_client(settings.fiat.timeout);
    PaybisClient::new(
        ReqwestClient::new(settings.fiat.paybis.url, client),
        settings.fiat.paybis.key.public,
        settings.fiat.paybis.key.secret,
    )
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_banxa_test_client() -> BanxaClient {
    let settings = get_test_settings();
    let client = crate::request_client(settings.fiat.timeout);
    BanxaClient::new(
        ReqwestClient::new(settings.fiat.banxa.api.url, client),
        settings.fiat.banxa.redirect.url,
        settings.fiat.banxa.partner,
        settings.fiat.banxa.key.secret,
        settings.fiat.banxa.webhook.key.secret,
    )
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_mercuryo_test_client() -> MercuryoClient {
    let settings = get_test_settings();
    let client = crate::request_client(settings.fiat.timeout);
    MercuryoClient::new(
        ReqwestClient::new(settings.fiat.mercuryo.url, client),
        settings.fiat.mercuryo.key.public,
        settings.fiat.mercuryo.key.secret,
        settings.fiat.mercuryo.webhook.key.secret,
    )
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
impl FiatMapping {
    pub fn mock() -> Self {
        FiatMapping {
            asset: primitives::Asset::from_chain(primitives::Chain::Bitcoin),
            asset_symbol: primitives::FiatAssetSymbol {
                symbol: "BTC".to_string(),
                network: Some("BITCOIN".to_string()),
            },
            unsupported_countries: std::collections::HashMap::new(),
            buy_limits: vec![],
            sell_limits: vec![],
        }
    }
}
