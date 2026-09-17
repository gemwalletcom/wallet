use diesel::prelude::*;
use primitives::{AddressName, Chain, ScanAddress, VerificationStatus};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AddressType, ChainRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddressRow {
    pub id: i32,
    pub chain: ChainRow,
    pub address: String,
    pub name: Option<String>,
    #[diesel(column_name = type_)]
    pub type_: AddressType,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

impl ScanAddressRow {
    pub fn is_verified_for(&self, chain: Chain, address: &str) -> bool {
        self.chain.0 == chain && self.address == address && self.is_verified && !self.is_fraudulent
    }

    pub fn as_primitive(self) -> Option<AddressName> {
        Some(AddressName {
            chain: self.chain.0,
            address: self.address,
            name: self.name?,
            address_type: self.type_.0.clone(),
            status: if self.is_fraudulent {
                VerificationStatus::Suspicious
            } else if self.is_verified {
                VerificationStatus::Verified
            } else {
                VerificationStatus::Unverified
            },
            image_url: None,
        })
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewScanAddressRow {
    pub chain: ChainRow,
    pub address: String,
    pub name: Option<String>,
    #[diesel(column_name = type_)]
    pub type_: AddressType,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
}

impl NewScanAddressRow {
    pub fn from_primitive(scan_address: ScanAddress) -> Self {
        Self {
            chain: ChainRow::from(scan_address.chain),
            address: scan_address.address,
            name: scan_address.name,
            type_: scan_address.address_type.unwrap_or(primitives::AddressType::Address).into(),
            is_verified: scan_address.is_verified.unwrap_or(false),
            is_fraudulent: scan_address.is_malicious.unwrap_or(false),
            is_memo_required: scan_address.is_memo_required.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScanAddressRow;
    use primitives::Chain;

    #[test]
    fn as_primitive_returns_none_without_name() {
        let row = ScanAddressRow::mock(1, Chain::Ethereum, "0x0000000000000000000000000000000000000001", None);

        assert_eq!(row.as_primitive(), None);
    }

    #[test]
    fn test_is_verified_for() {
        let mut row = ScanAddressRow::mock(1, Chain::Arbitrum, "0xAbC", None);
        assert!(!row.is_verified_for(Chain::Arbitrum, "0xAbC"));

        row.is_verified = true;
        assert!(row.is_verified_for(Chain::Arbitrum, "0xAbC"));
        assert!(!row.is_verified_for(Chain::Arbitrum, "0xabc"));
        assert!(!row.is_verified_for(Chain::Arbitrum, "0xABC"));
        assert!(!row.is_verified_for(Chain::Ethereum, "0xAbC"));
        assert!(!row.is_verified_for(Chain::Arbitrum, "0x456"));

        row.chain = Chain::Solana.into();
        assert!(!row.is_verified_for(Chain::Solana, "0xabc"));
        assert!(row.is_verified_for(Chain::Solana, "0xAbC"));
        row.chain = Chain::Arbitrum.into();

        row.is_fraudulent = true;
        assert!(!row.is_verified_for(Chain::Arbitrum, "0xAbC"));
    }
}
