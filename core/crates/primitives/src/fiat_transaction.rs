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

impl FiatTransactionStatus {
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Complete | Self::Failed => true,
            Self::Pending | Self::Unknown => false,
        }
    }

    pub fn merged_with(self, updated: Self) -> Self {
        let replaces = match self {
            Self::Unknown => true,
            Self::Pending => updated != Self::Unknown,
            Self::Complete | Self::Failed => updated.is_completed(),
        };
        if replaces { updated } else { self }
    }
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
    fn test_transaction_currency_is_kept_as_a_provider_string() {
        for currency in ["RON", "GEL", "AMD", "MDL", "UNKNOWN"] {
            let transaction = FiatTransaction {
                fiat_currency: currency.to_string(),
                ..FiatTransaction::mock()
            };
            let value = serde_json::to_value(&transaction).unwrap();

            assert_eq!(value["fiatCurrency"], currency);
            assert_eq!(serde_json::from_value::<FiatTransaction>(value).unwrap().fiat_currency, currency);
        }
    }

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

    #[test]
    fn test_status_merged_with() {
        use FiatTransactionStatus::*;

        assert_eq!(Complete.merged_with(Pending), Complete);
        assert_eq!(Complete.merged_with(Unknown), Complete);
        assert_eq!(Complete.merged_with(Failed), Failed);
        assert_eq!(Failed.merged_with(Pending), Failed);
        assert_eq!(Failed.merged_with(Complete), Complete);
        assert_eq!(Pending.merged_with(Complete), Complete);
        assert_eq!(Pending.merged_with(Unknown), Pending);
        assert_eq!(Unknown.merged_with(Pending), Pending);
    }
}
