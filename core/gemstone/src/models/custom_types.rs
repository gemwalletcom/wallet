use chrono::{DateTime, Utc};
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, NFTAssetId, NFTCollectionId, PerpetualId, StakeChain, WalletId};
use std::str::FromStr;

uniffi::custom_type!(StakeChain, String, {
    remote,
    lower: |s| s.as_ref().to_string(),
    try_lift: |s| StakeChain::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid StakeChain")),
});

uniffi::custom_type!(WalletId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| WalletId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid WalletId")),
});

uniffi::custom_type!(AssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| AssetId::new(&s).ok_or_else(|| uniffi::deps::anyhow::Error::msg("Invalid AssetId")),
});

uniffi::custom_type!(NFTAssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| NFTAssetId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid NFTAssetId")),
});

uniffi::custom_type!(NFTCollectionId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| NFTCollectionId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid NFTCollectionId")),
});

uniffi::custom_type!(PerpetualId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| PerpetualId::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid PerpetualId")),
});

pub type GemBigInt = BigInt;
pub type GemBigUint = BigUint;

uniffi::custom_type!(GemBigInt, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigInt::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigInt")),
});

uniffi::custom_type!(GemBigUint, String, {
    remote,
    lower: |value| value.to_string(),
    try_lift: |s| BigUint::from_str(&s)
        .map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid BigUint")),
});

pub type DateTimeUtc = DateTime<Utc>;

uniffi::custom_type!(DateTimeUtc, i64, {
    remote,
    lower: |value: DateTimeUtc| value.timestamp(),
    try_lift: |timestamp| {
        DateTime::<Utc>::from_timestamp(timestamp, 0)
            .ok_or_else(|| uniffi::deps::anyhow::Error::msg("Invalid timestamp"))
    },
});

pub mod decimal_string {
    use super::GemBigInt;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S: Serializer>(value: &GemBigInt, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<GemBigInt, D::Error> {
        let text = String::deserialize(deserializer)?;
        GemBigInt::from_str(&text).map_err(serde::de::Error::custom)
    }

    pub mod unsigned {
        use super::super::GemBigUint;
        use serde::{Deserialize, Deserializer, Serializer};
        use std::str::FromStr;

        pub fn serialize<S: Serializer>(value: &GemBigUint, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&value.to_string())
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<GemBigUint, D::Error> {
            let text = String::deserialize(deserializer)?;
            GemBigUint::from_str(&text).map_err(serde::de::Error::custom)
        }
    }

    pub mod optional {
        use super::super::GemBigInt;
        use serde::{Deserialize, Deserializer, Serializer};
        use std::str::FromStr;

        pub fn serialize<S: Serializer>(value: &Option<GemBigInt>, serializer: S) -> Result<S::Ok, S::Error> {
            match value {
                Some(value) => serializer.serialize_some(&value.to_string()),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<GemBigInt>, D::Error> {
            Option::<String>::deserialize(deserializer)?
                .map(|text| GemBigInt::from_str(&text).map_err(serde::de::Error::custom))
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lift_big_int(value: &str) -> uniffi::Result<GemBigInt> {
        let mut buffer = Vec::new();
        <String as uniffi::Lower<crate::UniFfiTag>>::write(value.to_string(), &mut buffer);
        <GemBigInt as uniffi::Lift<crate::UniFfiTag>>::try_read(&mut buffer.as_slice())
    }

    fn lift_big_uint(value: &str) -> uniffi::Result<GemBigUint> {
        let mut buffer = Vec::new();
        <String as uniffi::Lower<crate::UniFfiTag>>::write(value.to_string(), &mut buffer);
        <GemBigUint as uniffi::Lift<crate::UniFfiTag>>::try_read(&mut buffer.as_slice())
    }

    #[test]
    fn test_a_malformed_big_integer_is_rejected_at_the_boundary_instead_of_reading_as_zero() {
        assert_eq!(lift_big_int("101").unwrap(), GemBigInt::from(101));
        assert_eq!(lift_big_int("-101").unwrap(), GemBigInt::from(-101));
        assert!(lift_big_int("").is_err());
        assert!(lift_big_int("not-a-number").is_err());
    }

    #[test]
    fn test_a_negative_or_malformed_unsigned_value_is_rejected_at_the_boundary() {
        assert_eq!(lift_big_uint("101").unwrap(), GemBigUint::from(101u32));
        assert!(lift_big_uint("-1").is_err());
        assert!(lift_big_uint("").is_err());
    }
}
