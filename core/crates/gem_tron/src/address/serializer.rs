use super::TronAddress;
use primitives::Address as _;
#[cfg(feature = "signer")]
use serde::Serializer;
use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.map(|addr| TronAddress::from_hex(&addr).map(|address| address.encode()).unwrap_or(addr)))
}

#[cfg(feature = "signer")]
pub(crate) mod hex_or_base58 {
    use super::*;
    use serde::de::Error as _;

    pub(crate) fn serialize<S>(address: &TronAddress, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(address.as_bytes()))
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<TronAddress, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        TronAddress::from_hex_or_base58(&value).ok_or_else(|| D::Error::custom("invalid Tron address"))
    }
}
