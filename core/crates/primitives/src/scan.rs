use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{AssetId, Chain, ChainAddress, TransactionType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ScanSource {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum ScanProvider {
    GoPlus,
    HashDit,
    Tronscan,
}

impl ScanProvider {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTransactionPayload {
    pub origin: ScanAddressTarget,
    pub target: ScanAddressTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTransaction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_malicious: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_memo_required: Option<bool>,
    #[serde(default)]
    pub is_scan_complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malicious_addresses: Option<Vec<ChainAddress>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malicious_assets: Option<Vec<AssetId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malicious_website: Option<String>,
}

impl ScanTransaction {
    pub fn disabled() -> Self {
        Self {
            is_malicious: Some(false),
            is_memo_required: None,
            is_scan_complete: false,
            malicious_addresses: None,
            malicious_assets: None,
            malicious_website: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAddressTarget {
    pub asset_id: AssetId,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "CaseIterable, Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum AddressType {
    Address,
    Contract,
    Validator,
    Contact,
    InternalWallet,
}

impl AddressType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAddress {
    pub chain: Chain,
    pub address: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub address_type: Option<AddressType>,
    pub is_malicious: Option<bool>,
    pub is_memo_required: Option<bool>,
    pub is_verified: Option<bool>,
}

impl ScanAddress {
    pub fn contract(chain: Chain, address: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            chain,
            address: address.into(),
            name: Some(name.into()),
            address_type: Some(AddressType::Contract),
            is_malicious: Some(false),
            is_memo_required: Some(false),
            is_verified: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ScanTransactionPayload;

    #[test]
    fn test_scan_transaction_payload_optional_website() {
        for website in [None, Some("https://gemwallet.com/")] {
            let mut payload: ScanTransactionPayload = serde_json::from_str(include_str!("../testdata/scan_transaction_payload.json")).unwrap();
            payload.website = website.map(str::to_string);

            let serialized = serde_json::to_value(&payload).unwrap();
            let expected = website.map(|website| Value::String(website.to_string()));
            assert_eq!(serialized.get("website"), expected.as_ref());

            let decoded: ScanTransactionPayload = serde_json::from_value(serialized).unwrap();
            assert_eq!(decoded.website.as_deref(), website);
        }
    }
}
