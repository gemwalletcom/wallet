use crate::{Asset, AssetId, FiatProviderName, FiatQuoteUrlData};
use chrono::{DateTime, Utc};
use model_derive::Model;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransaction {
    pub id: String,
    pub asset_id: AssetId,
    pub transaction_type: FiatQuoteType,
    pub provider: FiatProviderName,
    #[model(skip)]
    #[serde(skip_serializing)]
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatus,
    #[model(skip)]
    #[serde(skip_serializing)]
    pub country: Option<String>,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
    #[model(skip)]
    #[serde(skip_serializing)]
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FiatTransaction {
    pub fn new_pending(data: &FiatQuoteUrlData, country: Option<String>, provider_transaction_id: Option<String>) -> Self {
        let quote = &data.quote;
        let now = Utc::now();

        Self {
            id: quote.id.clone(),
            asset_id: quote.asset.id.clone(),
            transaction_type: quote.quote_type,
            provider: quote.provider.id,
            provider_transaction_id,
            status: FiatTransactionStatus::Pending,
            country,
            fiat_amount: quote.fiat_amount,
            fiat_currency: quote.fiat_currency.clone(),
            value: quote.value.clone(),
            transaction_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionUpdate {
    pub transaction_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: FiatTransactionStatus,
    pub transaction_hash: Option<String>,
    pub fiat_amount: Option<f64>,
    pub fiat_currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum FiatWebhook {
    OrderId(String),
    Transaction(FiatTransactionUpdate),
    None,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionData {
    pub transaction: FiatTransaction,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatTransactionAssetData {
    pub id: String,
    pub asset: Asset,
    pub transaction_type: FiatQuoteType,
    pub provider: FiatProviderName,
    pub status: FiatTransactionStatus,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
    pub created_at: DateTime<Utc>,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, EnumString, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatTransactionStatus {
    Complete,
    Pending,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatQuoteType {
    Buy,
    Sell,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_transaction_data_json_decodes_back() {
        let data = FiatTransactionData {
            transaction: FiatTransaction::mock(),
            details_url: None,
        };
        let value = serde_json::to_value(&data).unwrap();

        assert_eq!(
            value,
            json!({
                "transaction": {
                    "id": "quote_123",
                    "assetId": "bitcoin",
                    "transactionType": "buy",
                    "provider": "moonpay",
                    "status": "pending",
                    "fiatAmount": 100.0,
                    "fiatCurrency": "USD",
                    "value": "100000",
                    "createdAt": "1970-01-01T00:00:00Z",
                    "updatedAt": "1970-01-01T00:00:00Z"
                },
                "detailsUrl": null
            })
        );
        assert_eq!(
            serde_json::from_value::<FiatTransactionData>(value).unwrap().transaction,
            FiatTransaction {
                provider_transaction_id: None,
                country: None,
                ..FiatTransaction::mock()
            }
        );
    }
}
