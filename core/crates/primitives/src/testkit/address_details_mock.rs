use num_bigint::BigUint;

use crate::{AddressDetails, AddressDetailsBalances, AddressType, AssetBalance, AssetId, Chain, VerificationStatus};

impl AddressDetails {
    pub fn mock() -> Self {
        Self {
            chain: Chain::Ethereum,
            address: "0x1".to_string(),
            name: None,
            address_type: AddressType::Address,
            status: VerificationStatus::Unverified,
            balances: Some(AddressDetailsBalances::mock()),
        }
    }
}

impl AddressDetailsBalances {
    pub fn mock() -> Self {
        Self {
            coin: AssetBalance::new(AssetId::from_chain(Chain::Ethereum), BigUint::from(1_500_000_000_000_000_000u64)),
            staking: None,
        }
    }
}
