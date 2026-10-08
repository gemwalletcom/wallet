use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{SinkExt, StreamExt};
use gem_auth::{AUTHORIZATION_HEADER, build_device_auth_header, create_auth_hash, device_public_key};
use primitives::hex::encode_with_0x;
use primitives::testkit::signer_mock::{TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS};
use primitives::{AddressChains, AuthMessage, AuthNonce, Chain, Device, PriceAlert, WalletId, WalletSubscription, WalletSubscriptionChains};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use signer::{SignatureScheme, Signer};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const GOLDEN_FILE: &str = "testdata/contract.json";
const DEVICE_VERSION: &str = "2.114.60";
const JSON: &str = "application/json";
const BITCOIN_ADDRESS: &str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
const TRANSACTION_ID: &str = "ethereum_0x0000000000000000000000000000000000000000000000000000000000000001";

#[derive(Clone, Copy, PartialEq)]
enum Compare {
    Full,
    StatusOnly,
    Rejection,
}

#[derive(Clone)]
enum Auth {
    None,
    Device,
    DeviceWallet,
    Header(String),
    Bearer(String),
}

struct Case {
    name: &'static str,
    method: Method,
    path: String,
    auth: Auth,
    body: Option<Vec<u8>>,
    content_type: Option<&'static str>,
    headers: Vec<(&'static str, String)>,
    compare: Compare,
}

impl Case {
    fn new(name: &'static str, method: Method, path: &str) -> Self {
        Self {
            name,
            method,
            path: path.to_string(),
            auth: Auth::None,
            body: None,
            content_type: None,
            headers: Vec::new(),
            compare: Compare::Full,
        }
    }

    fn device(mut self) -> Self {
        self.auth = Auth::Device;
        self
    }

    fn wallet(mut self) -> Self {
        self.auth = Auth::DeviceWallet;
        self
    }

    fn auth(mut self, auth: Auth) -> Self {
        self.auth = auth;
        self
    }

    fn json(mut self, body: Value) -> Self {
        self.body = Some(serde_json::to_vec(&body).unwrap());
        self.content_type = Some(JSON);
        self
    }

    fn raw(mut self, body: Vec<u8>, content_type: Option<&'static str>) -> Self {
        self.body = Some(body);
        self.content_type = content_type;
        self
    }

    fn header(mut self, name: &'static str, value: &str) -> Self {
        self.headers.push((name, value.to_string()));
        self
    }

    fn volatile(mut self) -> Self {
        self.compare = Compare::StatusOnly;
        self
    }

    fn rejection(mut self) -> Self {
        self.compare = Compare::Rejection;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Observed {
    status: u16,
    content_type: Option<String>,
    body: Value,
}

struct Harness {
    base: String,
    http: Client,
    private_key: [u8; 32],
    device_id: String,
    wallet_id: String,
    admin_secret: Option<String>,
}

impl Harness {
    fn new() -> Self {
        let mut private_key = [0u8; 32];
        private_key[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        private_key[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        let device_id = hex::encode(device_public_key(&private_key).unwrap());
        Self {
            base: env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string()),
            http: Client::builder().no_proxy().timeout(Duration::from_secs(30)).build().unwrap(),
            private_key,
            device_id,
            wallet_id: WalletId::Multicoin(TEST_PRIVATE_KEY_ETHEREUM_ADDRESS.to_string()).to_string(),
            admin_secret: env::var("API_ADMIN_SECRET").ok().filter(|secret| !secret.is_empty()),
        }
    }

    fn sign(&self, method: &str, path: &str, wallet_id: &str, body: &[u8], timestamp_ms: u64) -> String {
        build_device_auth_header(&self.private_key, method, path, wallet_id, body, timestamp_ms).unwrap()
    }

    fn sign_now(&self, method: &str, path: &str, wallet_id: &str, body: &[u8]) -> String {
        self.sign(method, path, wallet_id, body, now_ms())
    }

    fn signed_path<'a>(&self, path: &'a str) -> &'a str {
        if path.starts_with("/v3/") { path } else { path.split('?').next().unwrap_or(path) }
    }

    fn device(&self) -> Value {
        let device = Device {
            id: self.device_id.clone(),
            version: DEVICE_VERSION.to_string(),
            ..Device::mock()
        };
        serde_json::to_value(device).unwrap()
    }

    async fn send(&self, case: &Case) -> Observed {
        let signed_path = self.signed_path(&case.path);
        let body = case.body.clone().unwrap_or_default();
        let mut request = self.http.request(case.method.clone(), format!("{}{}", self.base, case.path));
        let authorization = match &case.auth {
            Auth::None => None,
            Auth::Device => Some(self.sign_now(case.method.as_str(), signed_path, "", &body)),
            Auth::DeviceWallet => Some(self.sign_now(case.method.as_str(), signed_path, &self.wallet_id, &body)),
            Auth::Header(value) => Some(value.clone()),
            Auth::Bearer(secret) => Some(format!("Bearer {secret}")),
        };
        if let Some(authorization) = authorization {
            request = request.header(AUTHORIZATION_HEADER, authorization);
        }
        if let Some(content_type) = case.content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        for (name, value) in &case.headers {
            request = request.header(*name, value);
        }
        if case.body.is_some() {
            request = request.body(body);
        }
        let response = match request.send().await {
            Ok(response) => response,
            Err(_) if case.compare == Compare::Rejection => {
                return Observed {
                    status: StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
                    content_type: None,
                    body: Value::String("<rejected>".to_string()),
                };
            }
            Err(error) => panic!("{}: {error}", case.name),
        };
        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_string());
        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(_) if case.compare == Compare::Rejection => Default::default(),
            Err(error) => panic!("{}: {error}", case.name),
        };
        let body = self.normalize(status, content_type.as_deref(), &bytes, case.compare);
        let status = if case.compare == Compare::Rejection && status.is_client_error() { StatusCode::PAYLOAD_TOO_LARGE } else { status };
        Observed {
            status: status.as_u16(),
            content_type: if case.compare == Compare::Rejection { None } else { content_type },
            body,
        }
    }

    fn normalize(&self, status: StatusCode, content_type: Option<&str>, bytes: &[u8], compare: Compare) -> Value {
        if compare == Compare::StatusOnly {
            return Value::String("<volatile>".to_string());
        }
        if compare == Compare::Rejection {
            return Value::String("<rejected>".to_string());
        }
        let text = String::from_utf8_lossy(bytes).replace(&self.device_id, "<device_id>").replace(&self.wallet_id, "<wallet_id>");
        if content_type == Some(JSON) {
            let value: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));
            if status.is_success() { shape(&value) } else { value }
        } else if bytes.is_empty() {
            Value::Null
        } else if content_type == Some("text/plain") {
            Value::String(text.lines().take(1).collect())
        } else {
            Value::String(format!("<{} bytes>", bytes.len()))
        }
    }

    async fn auth_nonce(&self) -> AuthNonce {
        let path = "/v2/devices/auth/nonce";
        let response = self.http.get(format!("{}{path}", self.base)).header(AUTHORIZATION_HEADER, self.sign_now("GET", path, "", &[])).send().await.unwrap();
        let text = response.text().await.unwrap();
        serde_json::from_str(&text).unwrap_or_else(|error| panic!("auth nonce {text}: {error}"))
    }

    async fn wallet_signed(&self, data: Value) -> Value {
        let auth_nonce = self.auth_nonce().await;
        let message = AuthMessage {
            chain: Chain::Ethereum,
            address: TEST_PRIVATE_KEY_ETHEREUM_ADDRESS.to_string(),
            auth_nonce: auth_nonce.clone(),
        };
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &create_auth_hash(&message).hash, &TEST_PRIVATE_KEY).unwrap();
        json!({
            "auth": {
                "deviceId": self.device_id,
                "chain": "ethereum",
                "address": TEST_PRIVATE_KEY_ETHEREUM_ADDRESS,
                "nonce": auth_nonce.nonce,
                "signature": encode_with_0x(&signature),
            },
            "data": data,
        })
    }

    async fn stream_round_trip(&self, url: &str) -> Observed {
        let path = url.rsplit_once("/v").map(|(_, suffix)| format!("/v{suffix}")).unwrap();
        let path = path.as_str();
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert(AUTHORIZATION_HEADER, HeaderValue::from_str(&self.sign_now("GET", path, "", &[])).unwrap());
        let (mut socket, response) = connect_async(request).await.unwrap();
        socket.send(Message::text(json!({"type": "subscribePrices", "data": {"assets": ["ethereum"]}}).to_string())).await.unwrap();
        let event = tokio::time::timeout(Duration::from_secs(10), socket.next()).await.unwrap().unwrap().unwrap();
        let body = match event {
            Message::Text(text) => shape(&serde_json::from_str::<Value>(&text).unwrap()),
            other => Value::String(format!("{other:?}")),
        };
        socket.close(None).await.unwrap();
        Observed {
            status: response.status().as_u16(),
            content_type: None,
            body,
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn shape(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(map.iter().map(|(key, value)| (key.clone(), shape(value))).collect()),
        Value::Array(items) => Value::Array(items.first().map(shape).into_iter().collect()),
        Value::String(_) => Value::String("string".to_string()),
        Value::Number(_) => Value::String("number".to_string()),
        Value::Bool(_) | Value::Null => value.clone(),
    }
}

fn subscription(wallet_id: &str) -> Value {
    serde_json::to_value(WalletSubscription {
        wallet_id: wallet_id.parse().unwrap(),
        source: None,
        subscriptions: vec![AddressChains {
            address: TEST_PRIVATE_KEY_ETHEREUM_ADDRESS.to_string(),
            chains: vec![Chain::Ethereum, Chain::SmartChain],
        }],
    })
    .unwrap()
}

fn subscription_chains(wallet_id: &str) -> Value {
    serde_json::to_value(WalletSubscriptionChains {
        wallet_id: wallet_id.parse().unwrap(),
        chains: vec![Chain::SmartChain],
    })
    .unwrap()
}

fn updated_locale(device: &Value) -> Value {
    let mut device = device.clone();
    device["locale"] = Value::String("de".to_string());
    device
}

fn price_alert() -> Value {
    serde_json::to_value(PriceAlert::mock(Chain::Ethereum, Some(1000.0))).unwrap()
}

fn public_cases() -> Vec<Case> {
    vec![
        Case::new("root", Method::GET, "/"),
        Case::new("health", Method::GET, "/health"),
        Case::new("metrics", Method::GET, "/metrics"),
        Case::new("v1_config", Method::GET, "/v1/config"),
        Case::new("v1_asset", Method::GET, "/v1/assets/ethereum"),
        Case::new("v1_asset_currency", Method::GET, "/v1/assets/ethereum?currency=EUR"),
        Case::new("v1_asset_bad_currency", Method::GET, "/v1/assets/ethereum?currency=BAD"),
        Case::new("v1_asset_unknown", Method::GET, "/v1/assets/ethereum_0x0000000000000000000000000000000000000001"),
        Case::new("v1_assets", Method::POST, "/v1/assets").json(json!(["ethereum", "bitcoin", "solana"])),
        Case::new("v1_assets_currency", Method::POST, "/v1/assets?currency=EUR").json(json!(["ethereum"])),
        Case::new("v1_assets_no_content_type", Method::POST, "/v1/assets").raw(br#"["ethereum"]"#.to_vec(), None),
        Case::new("v1_assets_invalid_json", Method::POST, "/v1/assets").raw(b"{".to_vec(), Some(JSON)),
        Case::new("v1_assets_wrong_shape", Method::POST, "/v1/assets").json(json!({"ids": []})),
        Case::new("v1_assets_oversize", Method::POST, "/v1/assets").raw(vec![b'['; 17 * 1024 * 1024], Some(JSON)).rejection(),
        Case::new("v1_assets_search", Method::GET, "/v1/assets/search?query=eth&chains=ethereum&limit=3"),
        Case::new("v1_assets_search_no_query", Method::GET, "/v1/assets/search"),
        Case::new("v1_assets_search_long_query", Method::GET, &format!("/v1/assets/search?query={}", "a".repeat(129))),
        Case::new("v1_search", Method::GET, "/v1/search?query=btc&limit=200"),
        Case::new("v1_price", Method::GET, "/v1/prices/ethereum"),
        Case::new("v1_prices", Method::POST, "/v1/prices").json(json!({"currency": "USD", "assetIds": ["ethereum"]})),
        Case::new("v1_fiat_rates", Method::GET, "/v1/fiat_rates"),
        Case::new("v1_charts", Method::GET, "/v1/charts/ethereum?period=day&currency=USD"),
        Case::new("v1_charts_bad_period", Method::GET, "/v1/charts/ethereum?period=century"),
        Case::new("v1_fiat_assets_buy", Method::GET, "/v1/fiat/assets/buy"),
        Case::new("v1_fiat_assets_bad", Method::GET, "/v1/fiat/assets/rent"),
        Case::new("v1_swap_assets", Method::GET, "/v1/swap/assets"),
        Case::new("v1_markets", Method::GET, "/v1/markets"),
        Case::new("v1_rewards_leaderboard", Method::GET, "/v1/rewards/leaderboard"),
        Case::new("v1_fee_estimates", Method::GET, "/v1/chain/fee-estimates").volatile(),
        Case::new("v1_nft_asset_preview", Method::GET, "/v1/nft/assets/ethereum_0x0000000000000000000000000000000000000001::1/preview").volatile(),
        Case::new("v1_nft_asset_preview_bad", Method::GET, "/v1/nft/assets/not-an-id/preview"),
        Case::new("v1_nft_collection_preview", Method::GET, "/v1/nft/collections/ethereum_0x0000000000000000000000000000000000000001/preview").volatile(),
        Case::new("v1_near_intents_quote", Method::POST, "/v1/swaps/near_intents/quote").json(json!({})).volatile(),
        Case::new("v1_swaps_xyz_action", Method::POST, "/v1/swaps/swaps_xyz/action").json(json!({})),
        Case::new("v1_okx_quote", Method::POST, "/v1/swaps/providers/okx/v6/quote").json(json!({})),
        Case::new("v1_webhook_path_secret_unknown", Method::POST, "/v1/webhooks/fiat/unknown/secret").json(json!({})),
        Case::new("v1_webhook_bad_kind", Method::POST, "/v1/webhooks/other/sender/secret").json(json!({})),
        Case::new("v1_webhook_header_missing", Method::POST, "/v1/webhooks/fiat/unknown").json(json!({})),
        Case::new("v1_webhook_header_basic", Method::POST, "/v1/webhooks/fiat/unknown").json(json!({})).auth(Auth::Header("Basic abc".to_string())),
        Case::new("v1_webhook_header_unknown", Method::POST, "/v1/webhooks/fiat/unknown").json(json!({})).auth(Auth::Bearer("secret".to_string())),
        Case::new("unmatched_route", Method::GET, "/v1/nope"),
        Case::new("unmatched_root_segment", Method::GET, "/nope"),
        Case::new("wrong_method", Method::DELETE, "/v1/config"),
        Case::new("trailing_slash", Method::GET, "/v1/config/"),
        Case::new("options_root", Method::OPTIONS, "/v1/config"),
        Case::new("head_health", Method::HEAD, "/health"),
    ]
}

fn unauthenticated_device_cases(harness: &Harness) -> Vec<Case> {
    let path = "/v2/devices";
    let expired = harness.sign("GET", path, "", &[], now_ms() - 10 * 60 * 1000);
    let future = harness.sign("GET", path, "", &[], now_ms() + 10 * 60 * 1000);
    let other_path = harness.sign_now("GET", "/v2/devices/is_registered", "", &[]);
    let other_method = harness.sign_now("POST", path, "", &[]);
    let other_key = build_device_auth_header(&[7u8; 32], "GET", path, "", &[], now_ms()).unwrap();
    vec![
        Case::new("v2_device_missing_auth", Method::GET, path),
        Case::new("v2_device_bearer_auth", Method::GET, path).auth(Auth::Bearer("secret".to_string())),
        Case::new("v2_device_garbage_auth", Method::GET, path).auth(Auth::Header("Gem not-base64!".to_string())),
        Case::new("v2_device_short_payload", Method::GET, path).auth(Auth::Header("Gem YWJj".to_string())),
        Case::new("v2_device_expired", Method::GET, path).auth(Auth::Header(expired)),
        Case::new("v2_device_future", Method::GET, path).auth(Auth::Header(future)),
        Case::new("v2_device_signature_other_path", Method::GET, path).auth(Auth::Header(other_path)),
        Case::new("v2_device_signature_other_method", Method::GET, path).auth(Auth::Header(other_method)),
        Case::new("v2_device_unregistered", Method::GET, path).auth(Auth::Header(other_key)),
        Case::new("v2_is_registered_before", Method::GET, "/v2/devices/is_registered").device(),
        Case::new("v2_device_before_registration", Method::GET, path).device(),
        Case::new("v2_wallet_route_without_wallet", Method::GET, "/v2/devices/assets").device(),
        Case::new("v2_register_id_mismatch", Method::POST, path).device().json(json!(Device::mock())),
        Case::new("v2_register_no_content_type", Method::POST, path).device().raw(serde_json::to_vec(&harness.device()).unwrap(), None),
        Case::new("v2_register_invalid_json", Method::POST, path).device().raw(b"{".to_vec(), Some(JSON)),
    ]
}

fn registered_device_cases(harness: &Harness) -> Vec<Case> {
    let device = harness.device();
    let wallet = harness.wallet_id.clone();
    let tampered_body = harness.sign_now("POST", "/v2/devices/subscriptions", "", br#"[]"#);
    let oversize = vec![b'['; 1024 * 1024 + 1];
    vec![
        Case::new("v2_register", Method::POST, "/v2/devices").device().json(device.clone()),
        Case::new("v2_register_twice", Method::POST, "/v2/devices").device().json(device.clone()),
        Case::new("v2_is_registered_after", Method::GET, "/v2/devices/is_registered").device(),
        Case::new("v2_device", Method::GET, "/v2/devices").device(),
        Case::new("v2_device_update", Method::PUT, "/v2/devices").device().json(updated_locale(&device)),
        Case::new("v2_device_update_mismatch", Method::PUT, "/v2/devices").device().json(json!(Device::mock())),
        Case::new("v2_body_hash_tamper", Method::POST, "/v2/devices/subscriptions")
            .auth(Auth::Header(tampered_body))
            .json(json!([subscription(&wallet)])),
        Case::new("v2_device_json_oversize", Method::POST, "/v2/devices/subscriptions").device().raw(oversize, Some(JSON)).rejection(),
        Case::new("v2_subscriptions_add", Method::POST, "/v2/devices/subscriptions").device().json(json!([subscription(&wallet)])),
        Case::new("v2_subscriptions_add_invalid", Method::POST, "/v2/devices/subscriptions").device().json(json!([{"walletId": 1}])),
        Case::new("v2_subscriptions", Method::GET, "/v2/devices/subscriptions").device(),
        Case::new("v2_wallet_unknown", Method::GET, "/v2/devices/assets").auth(Auth::Header(harness.sign_now("GET", "/v2/devices/assets", "multicoin_0xunknown", &[]))),
        Case::new("v2_assets", Method::GET, "/v2/devices/assets").wallet(),
        Case::new("v2_assets_from", Method::GET, "/v2/devices/assets?from_timestamp=1700000000").wallet(),
        Case::new("v2_assets_bad_from", Method::GET, "/v2/devices/assets?from_timestamp=soon").wallet(),
        Case::new("v2_transactions", Method::GET, "/v2/devices/transactions?limit=5&offset=0").wallet(),
        Case::new("v2_transactions_asset", Method::GET, "/v2/devices/transactions?asset_id=ethereum&from_timestamp=1").wallet(),
        Case::new("v2_transactions_bad_limit", Method::GET, "/v2/devices/transactions?limit=many").wallet(),
        Case::new("v2_transaction_by_id", Method::GET, &format!("/v2/devices/transactions/{TRANSACTION_ID}")).wallet(),
        Case::new("v2_transaction_by_id_bad", Method::GET, "/v2/devices/transactions/not-an-id").wallet(),
        Case::new("v2_transaction_legacy", Method::GET, &format!("/v2/devices/transaction/{TRANSACTION_ID}")).device(),
        Case::new("v2_address_names", Method::POST, "/v2/devices/address_names")
            .device()
            .json(json!([{"chain": "ethereum", "address": TEST_PRIVATE_KEY_ETHEREUM_ADDRESS}])),
        Case::new("v2_address_details", Method::GET, &format!("/v2/devices/addresses/bitcoin/{BITCOIN_ADDRESS}")).device().volatile(),
        Case::new("v2_address_details_bad_chain", Method::GET, "/v2/devices/addresses/mars/abc").device(),
        Case::new("v2_nft_assets", Method::GET, "/v2/devices/nft_assets").wallet(),
        Case::new("v2_nft_asset", Method::GET, "/v2/devices/nft_assets/ethereum_0x0000000000000000000000000000000000000001::1").device().volatile(),
        Case::new("v2_nft_asset_bad", Method::GET, "/v2/devices/nft_assets/bad").device(),
        Case::new("v2_nft_refresh", Method::POST, "/v2/devices/nft_assets/ethereum_0x0000000000000000000000000000000000000001::1/refresh")
            .wallet()
            .volatile(),
        Case::new("v2_nft_report", Method::POST, "/v2/devices/nft/report")
            .device()
            .json(json!({"collectionId": "ethereum_0x0000000000000000000000000000000000000001", "assetId": "bad", "reason": "spam"})),
        Case::new("v2_defi_positions", Method::GET, "/v2/devices/defi/positions").wallet(),
        Case::new("v2_rewards", Method::GET, "/v2/devices/rewards").wallet(),
        Case::new("v2_rewards_events", Method::GET, "/v2/devices/rewards/events").wallet(),
        Case::new("v2_rewards_redemption", Method::GET, "/v2/devices/rewards/redemptions/nope").device(),
        Case::new("v2_name_resolve", Method::GET, "/v2/devices/name/resolve/vitalik.eth?chain=ethereum").device().volatile(),
        Case::new("v2_name_resolve_no_chain", Method::GET, "/v2/devices/name/resolve/vitalik.eth").device(),
        Case::new("v2_scan_transaction", Method::POST, "/v2/devices/scan/transaction")
            .device()
            .json(json!({"origin": {"address": TEST_PRIVATE_KEY_ETHEREUM_ADDRESS, "chain": "ethereum"}, "target": {"address": TEST_PRIVATE_KEY_ETHEREUM_ADDRESS, "chain": "ethereum"}, "type": "transfer"}))
            .volatile(),
        Case::new("v2_wallet_configuration", Method::GET, "/v2/devices/wallet_configuration").wallet().volatile(),
        Case::new("v2_notifications", Method::GET, "/v2/devices/notifications?limit=10").device(),
        Case::new("v2_notifications_read", Method::POST, "/v2/devices/notifications/read").device(),
        Case::new("v2_notifications_read_as_get", Method::GET, "/v2/devices/notifications/read").device(),
        Case::new("v2_auth_nonce", Method::GET, "/v2/devices/auth/nonce").device(),
        Case::new("v2_token", Method::GET, "/v2/devices/token").device(),
        Case::new("v2_price_alerts_add", Method::POST, "/v2/devices/price_alerts").device().json(json!([price_alert()])),
        Case::new("v2_price_alerts", Method::GET, "/v2/devices/price_alerts").device(),
        Case::new("v2_price_alerts_asset", Method::GET, "/v2/devices/price_alerts?asset_id=ethereum").device(),
        Case::new("v2_price_alerts_delete", Method::DELETE, "/v2/devices/price_alerts").device().json(json!([price_alert()])),
        Case::new("v2_fiat_transactions", Method::GET, "/v2/devices/fiat/transactions").wallet(),
        Case::new("v2_fiat_assets", Method::GET, "/v2/devices/fiat/assets/sell").device(),
        Case::new("v2_fiat_quotes", Method::GET, "/v2/devices/fiat/quotes/buy/ethereum?amount=100&currency=USD").wallet().volatile(),
        Case::new("v2_fiat_quotes_no_amount", Method::GET, "/v2/devices/fiat/quotes/buy/ethereum").wallet(),
        Case::new("v2_fiat_quote_url", Method::GET, "/v2/devices/fiat/quotes/unknown-quote/url").wallet(),
        Case::new("v2_portfolio_assets", Method::POST, "/v2/devices/portfolio/assets?period=day")
            .device()
            .json(json!({"assets": [{"assetId": "ethereum", "value": "1000000000000000000"}]})),
        Case::new("v2_portfolio_assets_no_period", Method::POST, "/v2/devices/portfolio/assets").device().json(json!({"assets": []})),
        Case::new("v2_support_messages", Method::GET, "/v2/devices/support/messages").device().volatile(),
        Case::new("v2_support_image_no_content_type", Method::POST, "/v2/devices/support/messages/images?file_name=a.png")
            .device()
            .raw(vec![0u8; 16], None),
        Case::new("v2_support_image_too_small", Method::POST, "/v2/devices/support/messages/images?file_name=a.png")
            .device()
            .raw(vec![0u8; 16], Some("image/png")),
        Case::new("v2_push_notification", Method::POST, "/v2/devices/push-notification").device().volatile(),
        Case::new("v2_subscriptions_delete", Method::DELETE, "/v2/devices/subscriptions").device().json(json!([subscription_chains(&wallet)])),
        Case::new("v2_subscriptions_after_delete", Method::GET, "/v2/devices/subscriptions").device(),
    ]
}

fn v3_cases(harness: &Harness) -> Vec<Case> {
    let wallet = harness.wallet_id.clone();
    let query_signed_for_other_asset = harness.sign_now("GET", "/v3/devices/transactions?asset_id=bitcoin&limit=5", &wallet, &[]);
    let path_only_signature = harness.sign_now("GET", "/v3/devices/transactions", &wallet, &[]);
    let replayed = harness.sign_now("POST", "/v3/devices/notifications/read", "", &[]);
    vec![
        Case::new("v3_device", Method::GET, "/v3/devices").device(),
        Case::new("v3_is_registered", Method::GET, "/v3/devices/is-registered").device(),
        Case::new("v3_subscriptions", Method::GET, "/v3/devices/subscriptions").device(),
        Case::new("v3_transactions", Method::GET, "/v3/devices/transactions?asset_id=ethereum&limit=5").wallet(),
        Case::new("v3_transactions_query_tampered", Method::GET, "/v3/devices/transactions?asset_id=ethereum&limit=5").auth(Auth::Header(query_signed_for_other_asset)),
        Case::new("v3_transactions_path_only_signature", Method::GET, "/v3/devices/transactions?asset_id=ethereum").auth(Auth::Header(path_only_signature)),
        Case::new("v3_price_alerts", Method::GET, "/v3/devices/price-alerts?asset_id=ethereum").device(),
        Case::new("v3_wallet_configuration", Method::GET, "/v3/devices/wallet-configuration").wallet().volatile(),
        Case::new("v3_nft_assets", Method::GET, "/v3/devices/nft-assets").wallet(),
        Case::new("v3_names", Method::GET, "/v3/devices/names/vitalik.eth?chain=ethereum").device().volatile(),
        Case::new("v3_address_names", Method::POST, "/v3/devices/address-names")
            .device()
            .json(json!([{"chain": "ethereum", "address": TEST_PRIVATE_KEY_ETHEREUM_ADDRESS}])),
        Case::new("v3_notifications_read", Method::POST, "/v3/devices/notifications/read").auth(Auth::Header(replayed.clone())),
        Case::new("v3_notifications_read_replayed", Method::POST, "/v3/devices/notifications/read").auth(Auth::Header(replayed)),
        Case::new("v3_subscriptions_repeat_get", Method::GET, "/v3/devices/subscriptions").device(),
        Case::new("v3_dropped_token", Method::GET, "/v3/devices/token").device(),
        Case::new("v3_dropped_legacy_transaction", Method::GET, &format!("/v3/devices/transaction/{TRANSACTION_ID}")).device(),
        Case::new("v3_underscore_path", Method::GET, "/v3/devices/price_alerts").device(),
    ]
}

fn admin_cases(harness: &Harness) -> Vec<Case> {
    let device_id = harness.device_id.clone();
    let mut cases = vec![
        Case::new("admin_device_no_auth", Method::GET, &format!("/v1/admin/devices/{device_id}")),
        Case::new("admin_device_basic_auth", Method::GET, &format!("/v1/admin/devices/{device_id}")).auth(Auth::Header("Basic abc".to_string())),
        Case::new("admin_device_bad_secret", Method::GET, &format!("/v1/admin/devices/{device_id}")).auth(Auth::Bearer("not-a-secret".to_string())),
        Case::new("admin_asset_add_no_auth", Method::POST, "/v1/admin/assets/add").json(json!("ethereum")),
        Case::new("admin_chain_fee_no_auth", Method::GET, "/v1/admin/chain/fee-estimates/ethereum"),
        Case::new("admin_block_latest_no_auth", Method::GET, "/v1/admin/chain/blocks/ethereum/latest"),
        Case::new("admin_swap_quote_no_auth", Method::GET, "/v1/admin/chain/swaps/quote"),
        Case::new("admin_fiat_quotes_no_auth", Method::GET, "/v1/admin/fiat/quotes/buy?asset_id=ethereum&amount=100"),
        Case::new("admin_transactions_hash_no_auth", Method::GET, "/v1/admin/transactions/0x01"),
    ];
    if let Some(secret) = &harness.admin_secret {
        let bearer = Auth::Bearer(secret.clone());
        cases.extend([
            Case::new("admin_device", Method::GET, &format!("/v1/admin/devices/{device_id}")).auth(bearer.clone()),
            Case::new("admin_device_unknown", Method::GET, "/v1/admin/devices/unknown").auth(bearer.clone()),
            Case::new("admin_device_subscriptions", Method::GET, &format!("/v1/admin/devices/{device_id}/subscriptions")).auth(bearer.clone()),
            Case::new("admin_device_wallet_subscriptions", Method::GET, &format!("/v1/admin/devices/{device_id}/wallets/{}/subscriptions", harness.wallet_id)).auth(bearer.clone()),
            Case::new("admin_device_transactions", Method::GET, &format!("/v1/admin/devices/{device_id}/transactions")).auth(bearer.clone()),
            Case::new("admin_device_fiat_transactions", Method::GET, &format!("/v1/admin/devices/{device_id}/fiat/transactions")).auth(bearer.clone()),
            Case::new("admin_transactions_hash", Method::GET, "/v1/admin/transactions/0x01").auth(bearer.clone()),
            Case::new("admin_chain_fee", Method::GET, "/v1/admin/chain/fee-estimates/ethereum").auth(bearer.clone()).volatile(),
            Case::new("admin_chain_bad", Method::GET, "/v1/admin/chain/fee-estimates/mars").auth(bearer.clone()),
            Case::new("admin_asset_status_native", Method::POST, "/v1/admin/assets/status").auth(bearer.clone()).json(json!("ethereum")),
            Case::new("admin_asset_add_no_content_type", Method::POST, "/v1/admin/assets/add").auth(bearer.clone()).raw(br#""ethereum""#.to_vec(), None),
        ]);
    }
    cases
}

async fn run(harness: &Harness, cases: Vec<Case>, observed: &mut BTreeMap<String, Observed>) {
    for case in cases {
        assert!(!observed.contains_key(case.name), "duplicate case {}", case.name);
        let result = harness.send(&case).await;
        observed.insert(case.name.to_string(), result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_api_contract() {
    let harness = Harness::new();
    let mut observed = BTreeMap::new();
    run(&harness, public_cases(), &mut observed).await;
    run(&harness, unauthenticated_device_cases(&harness), &mut observed).await;
    let registered = registered_device_cases(&harness);
    let (before_wallet, after_wallet): (Vec<Case>, Vec<Case>) = registered.into_iter().partition(|case| case.name != "v2_subscriptions_delete" && case.name != "v2_subscriptions_after_delete");
    run(&harness, before_wallet, &mut observed).await;
    let referral = harness.wallet_signed(json!({"code": format!("c{}", &harness.device_id[..8])})).await;
    let redeem = harness.wallet_signed(json!({"id": "unknown-option"})).await;
    let reused_nonce = referral.clone();
    let mismatch = {
        let mut value = harness.wallet_signed(json!({"code": "x"})).await;
        value["auth"]["deviceId"] = Value::String("0".repeat(64));
        value
    };
    run(
        &harness,
        vec![
            Case::new("v2_referral_create", Method::POST, "/v2/devices/rewards/referrals/create").device().json(referral).volatile(),
            Case::new("v2_referral_create_reused_nonce", Method::POST, "/v2/devices/rewards/referrals/create").device().json(reused_nonce),
            Case::new("v2_referral_device_mismatch", Method::POST, "/v2/devices/rewards/referrals/create").device().json(mismatch),
            Case::new("v2_referral_use_no_user_agent", Method::POST, "/v2/devices/rewards/referrals/use")
                .device()
                .json(json!({"auth": {}, "data": {}}))
                .header("User-Agent", ""),
            Case::new("v2_referral_use_invalid_body", Method::POST, "/v2/devices/rewards/referrals/use")
                .device()
                .json(json!({"auth": {}, "data": {}}))
                .header("User-Agent", "contract-test"),
            Case::new("v2_rewards_redeem", Method::POST, "/v2/devices/rewards/redeem").wallet().json(redeem).volatile(),
        ],
        &mut observed,
    )
    .await;
    run(&harness, admin_cases(&harness), &mut observed).await;
    run(&harness, v3_cases(&harness), &mut observed).await;
    run(&harness, after_wallet, &mut observed).await;
    if let Ok(url) = env::var("API_WS_URL") {
        observed.insert("ws_stream_subscribe_prices".to_string(), harness.stream_round_trip(&format!("{url}/v2/devices/stream")).await);
        observed.insert("ws_v3_stream_subscribe_prices".to_string(), harness.stream_round_trip(&format!("{url}/v3/devices/stream")).await);
        observed.insert("ws_health".to_string(), harness.send(&Case::new("ws_health", Method::GET, "/health")).await);
    }

    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_FILE);
    if env::var("CONTRACT_RECORD").is_ok() {
        fs::write(&golden_path, serde_json::to_string_pretty(&observed).unwrap() + "\n").unwrap();
        return;
    }
    let golden: BTreeMap<String, Observed> = serde_json::from_str(&fs::read_to_string(&golden_path).unwrap()).unwrap();
    let mut differences = Vec::new();
    for (name, expected) in &golden {
        match observed.get(name) {
            Some(actual) if actual == expected => {}
            Some(actual) => differences.push(format!("{name}: expected {} got {}", serde_json::to_string(expected).unwrap(), serde_json::to_string(actual).unwrap())),
            None => differences.push(format!("{name}: missing")),
        }
    }
    for name in observed.keys().filter(|name| !golden.contains_key(*name)) {
        differences.push(format!("{name}: not in golden file"));
    }
    assert!(differences.is_empty(), "{} contract differences:\n{}", differences.len(), differences.join("\n"));
}
