use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FonbnkCurrencyType {
    Fiat,
    Crypto,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FonbnkPaymentChannel {
    MobileMoney,
    Bank,
    Crypto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkCurrency {
    pub currency_type: String,
    pub currency_code: String,
    pub payment_channels: Vec<FonbnkCurrencyChannel>,
    pub currency_details: FonbnkCurrencyDetails,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkCurrencyChannel {
    #[serde(rename = "type")]
    pub channel: String,
    pub is_deposit_allowed: bool,
    pub is_payout_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkCurrencyDetails {
    pub country_iso_code: Option<String>,
    pub network: Option<String>,
    pub asset: Option<String>,
    pub contract_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkQuoteLeg {
    pub payment_channel: FonbnkPaymentChannel,
    pub currency_type: FonbnkCurrencyType,
    pub currency_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_iso_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
}

impl FonbnkQuoteLeg {
    pub fn fiat(currency_code: String, country_iso_code: String, payment_channel: FonbnkPaymentChannel) -> Self {
        Self {
            payment_channel,
            currency_type: FonbnkCurrencyType::Fiat,
            currency_code,
            country_iso_code: Some(country_iso_code),
            amount: None,
        }
    }

    pub fn crypto(currency_code: String) -> Self {
        Self {
            payment_channel: FonbnkPaymentChannel::Crypto,
            currency_type: FonbnkCurrencyType::Crypto,
            currency_code,
            country_iso_code: None,
            amount: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FonbnkQuoteRequest {
    pub deposit: FonbnkQuoteLeg,
    pub payout: FonbnkQuoteLeg,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkOrderLimitsQuery {
    deposit_payment_channel: FonbnkPaymentChannel,
    deposit_currency_type: FonbnkCurrencyType,
    deposit_currency_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    deposit_country_iso_code: Option<String>,
    payout_payment_channel: FonbnkPaymentChannel,
    payout_currency_type: FonbnkCurrencyType,
    payout_currency_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payout_country_iso_code: Option<String>,
}

impl FonbnkOrderLimitsQuery {
    pub fn new(deposit: &FonbnkQuoteLeg, payout: &FonbnkQuoteLeg) -> Self {
        Self {
            deposit_payment_channel: deposit.payment_channel,
            deposit_currency_type: deposit.currency_type,
            deposit_currency_code: deposit.currency_code.clone(),
            deposit_country_iso_code: deposit.country_iso_code.clone(),
            payout_payment_channel: payout.payment_channel,
            payout_currency_type: payout.currency_type,
            payout_currency_code: payout.currency_code.clone(),
            payout_country_iso_code: payout.country_iso_code.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkOrderLimits {
    pub deposit: FonbnkLegLimits,
    pub payout: FonbnkLegLimits,
    pub supported: bool,
    pub unsupported_reason: Option<FonbnkUnsupportedReason>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkLegLimits {
    pub min: f64,
    pub max: f64,
    pub min_usd: f64,
    pub max_usd: f64,
    pub step: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkUnsupportedReason {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkQuote {
    pub quote_id: String,
    pub deposit: FonbnkQuoteLegResponse,
    pub payout: FonbnkQuoteLegResponse,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkQuoteLegResponse {
    pub cashout: FonbnkCashout,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkCashout {
    pub amount_before_fees: f64,
    pub amount_after_fees: f64,
    pub amount_before_fees_usd: f64,
    pub amount_after_fees_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FonbnkWebhook {
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FonbnkOrderWebhookData {
    pub order: FonbnkOrder,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkOrder {
    #[serde(rename = "_id")]
    pub id: String,
    pub merchant_order_params: Option<String>,
    #[serde(rename = "type")]
    pub order_type: String,
    pub status: String,
    pub deposit: FonbnkOrderLeg,
    pub payout: FonbnkOrderLeg,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkOrderLeg {
    pub currency_code: String,
    pub cashout: FonbnkOrderCashout,
    pub transaction: Option<FonbnkOrderTransaction>,
}

impl FonbnkOrderLeg {
    pub fn transaction_hash(&self) -> Option<String> {
        self.transaction.as_ref()?.meta.as_ref()?.transaction_hash.clone()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkOrderCashout {
    pub amount_before_fees: f64,
    pub amount_after_fees: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FonbnkOrderTransaction {
    pub meta: Option<FonbnkTransactionMeta>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FonbnkTransactionMeta {
    pub transaction_hash: Option<String>,
}
