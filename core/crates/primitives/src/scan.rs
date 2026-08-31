use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{AssetId, Chain, ChainAddress, TransactionType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanTransactionPayload {
    pub origin: ScanAddressTarget,
    pub target: ScanAddressTarget,
    pub website: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanTransaction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_malicious: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_memo_required: Option<bool>,
    #[serde(default)]
    #[typeshare(skip)]
    pub is_scan_complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub malicious_addresses: Option<Vec<ChainAddress>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub malicious_assets: Option<Vec<AssetId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub malicious_website: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
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
