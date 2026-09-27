use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::{AddressName, AssetId, Chain, ChainAddress, TransactionType, VerificationStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ScanSource {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRefStr, IntoStaticStr, EnumIter, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ScanType {
    Address,
    AddressPoisoning,
    Website,
    Asset,
}

impl ScanType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn is_safe_cacheable(&self) -> bool {
        matches!(self, Self::Address | Self::Website)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr, IntoStaticStr, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ScanOutcome {
    Clean,
    Malicious,
    Pending,
    Error,
}

impl ScanOutcome {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAddressTarget {
    pub asset_id: AssetId,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, Model)]
#[model(swift = "CaseIterable, Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum AddressType {
    Address,
    Contract,
    Asset,
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
    pub fn is_verified_for(&self, chain: Chain, address: &str) -> bool {
        self.chain == chain && self.address == address && self.is_verified == Some(true) && self.is_malicious != Some(true)
    }

    pub fn verification_status(&self) -> VerificationStatus {
        if self.is_malicious == Some(true) {
            VerificationStatus::Suspicious
        } else if self.is_verified == Some(true) {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Unverified
        }
    }

    pub fn address_name(&self) -> Option<AddressName> {
        Some(AddressName {
            chain: self.chain,
            address: self.address.clone(),
            name: self.name.clone()?,
            address_type: self.address_type.clone()?,
            status: self.verification_status(),
            image_url: None,
        })
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanVerdict {
    pub scan_type: ScanType,
    pub chain: Option<Chain>,
    pub target: String,
    pub provider: ScanProvider,
    pub reason: Option<String>,
}

impl ScanVerdict {
    pub fn matches(&self, scan_type: ScanType, chain: Option<Chain>, target: &str) -> bool {
        self.scan_type == scan_type && self.chain == chain && self.target == target
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{AddressName, AddressType, Chain, ScanAddress, ScanProvider, ScanTransactionPayload, ScanType, ScanVerdict, VerificationStatus};

    #[test]
    fn test_scan_address_address_name() {
        let router = ScanAddress::contract(Chain::Arbitrum, "0xAbC", "Router");
        assert_eq!(
            router.address_name(),
            Some(AddressName {
                chain: Chain::Arbitrum,
                address: "0xAbC".to_string(),
                name: "Router".to_string(),
                address_type: AddressType::Contract,
                status: VerificationStatus::Verified,
                image_url: None,
            })
        );
        assert_eq!(ScanAddress { name: None, ..router.clone() }.address_name(), None);
        assert_eq!(ScanAddress { is_malicious: Some(true), ..router.clone() }.address_name().map(|name| name.status), Some(VerificationStatus::Suspicious));
        assert_eq!(ScanAddress { is_verified: Some(false), ..router }.address_name().map(|name| name.status), Some(VerificationStatus::Unverified));
    }

    #[test]
    fn test_scan_address_is_verified_for() {
        let mut address = ScanAddress::contract(Chain::Arbitrum, "0xAbC", "Router");
        assert!(address.is_verified_for(Chain::Arbitrum, "0xAbC"));
        assert!(!address.is_verified_for(Chain::Arbitrum, "0xabc"));
        assert!(!address.is_verified_for(Chain::Ethereum, "0xAbC"));

        address.is_malicious = Some(true);
        assert!(!address.is_verified_for(Chain::Arbitrum, "0xAbC"));

        address.is_malicious = Some(false);
        address.is_verified = Some(false);
        assert!(!address.is_verified_for(Chain::Arbitrum, "0xAbC"));
    }

    #[test]
    fn test_scan_verdict_matches() {
        let verdict = ScanVerdict {
            scan_type: ScanType::Address,
            chain: Some(Chain::SmartChain),
            target: "0x123".to_string(),
            provider: ScanProvider::HashDit,
            reason: None,
        };

        assert!(verdict.matches(ScanType::Address, Some(Chain::SmartChain), "0x123"));
        assert!(!verdict.matches(ScanType::AddressPoisoning, Some(Chain::SmartChain), "0x123"));
        assert!(!verdict.matches(ScanType::Address, Some(Chain::Ethereum), "0x123"));
        assert!(!verdict.matches(ScanType::Address, None, "0x123"));
        assert!(!verdict.matches(ScanType::Address, Some(Chain::SmartChain), "0x456"));
    }

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
