use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Deserialize)]
pub struct TopperAssetsResponse {
    pub assets: TopperAssets,
}

#[derive(Debug, Deserialize)]
pub struct TopperAssets {
    pub target: Vec<TopperAsset>,
}

#[derive(Debug, Deserialize)]
pub struct TopperAsset {
    pub code: String,
    pub networks: Vec<TopperAssetNetwork>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopperAssetNetwork {
    pub code: String,
    pub asset_contract_address: Option<String>,
    pub asset_type: Option<TopperAssetType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TopperAssetType {
    Coin,
    Token,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct TopperCountriesResponse {
    pub countries: Vec<TopperCountry>,
}

#[derive(Debug, Deserialize)]
pub struct TopperCountry {
    pub alpha2: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopperPaymentMethodsResponse {
    pub payment_methods: Vec<TopperPaymentMethod>,
}

#[derive(Debug, Deserialize)]
pub struct TopperPaymentMethod {
    #[serde(rename = "type")]
    pub method_type: TopperPaymentMethodType,
    pub limits: Option<Vec<TopperLimit>>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TopperPaymentMethodType {
    CreditCard,
    DebitCard,
    ApplePay,
    GooglePay,
    Paypal,
    Sepa,
    Venmo,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct TopperLimit {
    pub asset: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub minimum: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub maximum: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopperSimulationRequest {
    pub bootstrap_token: String,
}

#[derive(Debug, Deserialize)]
pub struct TopperSimulationResponse {
    pub simulation: TopperSimulation,
}

#[derive(Debug, Deserialize)]
pub struct TopperSimulation {
    pub destination: TopperSimulationAmount,
}

#[derive(Debug, Deserialize)]
pub struct TopperSimulationAmount {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TopperOnRamp {
    pub source: TopperSource,
    pub target: TopperRecipient,
    pub simulation: TopperCountryHint,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TopperSource {
    pub amount: String,
    pub asset: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TopperRecipient {
    pub asset: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    pub recipient_edit_mode: TopperRecipientEditMode,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TopperRecipientEditMode {
    NotEditable,
    OnlyAddressAndTag,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TopperCountryHint {
    pub country: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopperWebhook {
    pub name: TopperEvent,
    pub bootstrap_token_id: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub enum TopperEvent {
    #[serde(rename = "order:crypto-onramp:committed")]
    Committed,
    #[serde(rename = "order:crypto-onramp:charged")]
    Charged,
    #[serde(rename = "order:crypto-onramp:crypto-sent")]
    CryptoSent,
    #[serde(rename = "order:crypto-onramp:completed")]
    Completed,
    #[serde(rename = "order:crypto-onramp:expired")]
    Expired,
    #[serde(rename = "order:crypto-onramp:failed")]
    Failed,
    #[serde(rename = "order:crypto-onramp:refund:completed")]
    Refunded,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct TopperOrder {
    pub id: String,
    pub origin: TopperOrderOrigin,
    pub destination: TopperOrderDestination,
}

#[derive(Debug, Deserialize)]
pub struct TopperOrderOrigin {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub asset: String,
}

#[derive(Debug, Deserialize)]
pub struct TopperOrderDestination {
    pub ledger: Option<TopperLedger>,
}

#[derive(Debug, Deserialize)]
pub struct TopperLedger {
    pub txid: String,
}
