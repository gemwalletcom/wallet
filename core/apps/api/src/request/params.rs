use std::fmt;
use std::str::FromStr;

pub(crate) use primitives::MAX_QUERY_LIMIT;
use primitives::currency::Currency;
use primitives::{AssetId, Chain, ChartPeriod, FiatProviderName, FiatQuoteType, NFTAssetId, NFTCollectionId, SwapProvider, TransactionId, WebhookKind};
use serde::de::{self, Deserialize, Deserializer};

const MAX_ADDRESS_LENGTH: usize = 256;
const MAX_ASSET_ID_LENGTH: usize = 256;
const MAX_NFT_ID_LENGTH: usize = 256;
const MAX_SEARCH_QUERY_LENGTH: usize = 128;

macro_rules! string_param {
    ($name:ident, $inner:ty, $label:literal, $parse:expr) => {
        pub struct $name(pub $inner);

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let parse: fn(&str) -> Option<$inner> = $parse;
                parse(value).map(Self).ok_or_else(|| format!(concat!("Invalid ", $label, ": {}"), value))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(de::Error::custom)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }
    };
}

pub fn lenient<'de, D: Deserializer<'de>, T: FromStr>(deserializer: D) -> Result<Option<T>, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?.and_then(|value| value.parse().ok()))
}

fn bounded(value: &str, max: usize) -> Option<&str> {
    (!value.is_empty() && value.len() <= max).then_some(value)
}

string_param!(ChainParam, Chain, "chain", |value| Chain::from_str(value).ok());
string_param!(TransactionIdParam, TransactionId, "transaction id", |value| TransactionId::from_str(value).ok());
string_param!(FiatQuoteTypeParam, FiatQuoteType, "quote type", |value| FiatQuoteType::from_str(value).ok());
string_param!(AddressParam, String, "address", |value| bounded(value, MAX_ADDRESS_LENGTH).map(str::to_string));
string_param!(NftCollectionIdParam, NFTCollectionId, "collection id", |value| bounded(value, MAX_NFT_ID_LENGTH).and_then(|value| value.parse().ok()));
string_param!(NftAssetIdParam, NFTAssetId, "nft asset id", |value| bounded(value, MAX_NFT_ID_LENGTH).and_then(|value| value.parse().ok()));
string_param!(AssetIdParam, AssetId, "asset_id", |value| bounded(value, MAX_ASSET_ID_LENGTH).and_then(AssetId::new));
string_param!(ChartPeriodParam, ChartPeriod, "period", |value| ChartPeriod::new(value.to_string()));
string_param!(SwapProviderParam, SwapProvider, "provider", |value| SwapProvider::from_str(value).ok());
string_param!(FiatProviderIdParam, FiatProviderName, "provider", |value| FiatProviderName::from_str(value).ok());
string_param!(SearchQueryParam, String, "query length", |value| (value.len() <= MAX_SEARCH_QUERY_LENGTH).then(|| value.to_string()));
string_param!(WebhookKindParam, WebhookKind, "webhook kind", |value| WebhookKind::from_str(value).ok());
string_param!(CurrencyParam, Currency, "currency", |value| Currency::from_str(value).ok());

impl Default for CurrencyParam {
    fn default() -> Self {
        Self(Currency::USD)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QueryLimitParam(pub usize);

impl Default for QueryLimitParam {
    fn default() -> Self {
        Self(MAX_QUERY_LIMIT)
    }
}

impl<'de> Deserialize<'de> for QueryLimitParam {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        let limit = usize::from_str(&value).map_err(|_| de::Error::custom(format!("Invalid limit: {value}")))?;
        Ok(Self(limit.min(MAX_QUERY_LIMIT)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Params {
        #[serde(default)]
        currency: CurrencyParam,
        #[serde(default)]
        limit: QueryLimitParam,
    }

    fn parse(query: &str) -> Result<Params, serde_urlencoded::de::Error> {
        serde_urlencoded::from_str(query)
    }

    #[test]
    fn test_query_params_default_and_validate() {
        let params = parse("").unwrap();
        assert_eq!(params.currency.0, Currency::USD);
        assert_eq!(params.limit.0, MAX_QUERY_LIMIT);

        let params = parse("currency=EUR&limit=50").unwrap();
        assert_eq!(params.currency.0, Currency::EUR);
        assert_eq!(params.limit.0, 50);

        assert_eq!(parse("limit=200").unwrap().limit.0, MAX_QUERY_LIMIT);
        assert!(parse("currency=BAD").is_err());
        assert!(parse("limit=invalid").is_err());
        assert!("a".repeat(129).parse::<SearchQueryParam>().is_err());
        assert!("".parse::<AddressParam>().is_err());
    }
}
