use serde::{Deserialize, Serialize};

use crate::{AssetBalance, Chain, VerificationStatus, scan::AddressType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddressDetails {
    pub chain: Chain,
    pub address: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub address_type: AddressType,
    pub status: VerificationStatus,
    pub balances: Option<AddressDetailsBalances>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddressDetailsBalances {
    pub coin: AssetBalance,
    pub staking: Option<AssetBalance>,
}
