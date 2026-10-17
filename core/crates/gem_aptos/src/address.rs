use gem_hash::sha3::sha3_256;
use primitives::{Address as AddressTrait, SignerError, decode_hex};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const ADDRESS_LENGTH: usize = 32;
const OBJECT_DERIVED_SCHEME: u8 = 0xFC;
const OBJECT_FROM_SEED_SCHEME: u8 = 0xFE;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub const fn new(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn from_hex(value: &str) -> Result<Self, SignerError> {
        <Self as FromStr>::from_str(value)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignerError> {
        if bytes.len() > ADDRESS_LENGTH {
            return Err(SignerError::InvalidInput("Aptos address too long".to_string()));
        }
        let mut address = [0u8; ADDRESS_LENGTH];
        let offset = ADDRESS_LENGTH - bytes.len();
        address[offset..].copy_from_slice(bytes);
        Ok(Self(address))
    }

    pub fn one() -> Self {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes[ADDRESS_LENGTH - 1] = 1;
        Self(bytes)
    }

    pub fn primary_store(&self, metadata: &AccountAddress) -> AccountAddress {
        Self(sha3_256(&[&self.0[..], &metadata.0[..], &[OBJECT_DERIVED_SCHEME]].concat()))
    }

    pub fn named_object(&self, seed: &[u8]) -> AccountAddress {
        Self(sha3_256(&[&self.0[..], seed, &[OBJECT_FROM_SEED_SCHEME]].concat()))
    }
}

impl FromStr for AccountAddress {
    type Err = SignerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = decode_hex(value)?;
        Self::from_bytes(&bytes)
    }
}

impl AddressTrait for AccountAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Self::from_hex(address).ok()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        self.to_string()
    }
}

pub fn validate_address(address: &str) -> bool {
    AccountAddress::is_valid(address)
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", ::hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Address;

    const VALID_ADDRESS: &str = "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc";

    #[test]
    fn test_aptos_address() {
        let padded = "0x07968dab936c1bad187c60ce4082f307d030d780e91e694ae03aef16aba73f30";
        let unpadded = "0x7968dab936c1bad187c60ce4082f307d030d780e91e694ae03aef16aba73f30";
        let reported_padded = "0x0638761ddc13e58b60aaa6f817fca9984b795f238fa46778e03af6859d72bc3d";
        let reported_unpadded = "0x638761ddc13e58b60aaa6f817fca9984b795f238fa46778e03af6859d72bc3d";
        let framework_address = format!("0x{}", "00".repeat(31) + "01");
        let non_special_short_address = format!("0x{}", "00".repeat(31) + "10");

        for (input, expected) in [
            (VALID_ADDRESS, VALID_ADDRESS.to_string()),
            (padded, padded.to_string()),
            (unpadded, padded.to_string()),
            (reported_padded, reported_padded.to_string()),
            (reported_unpadded, reported_padded.to_string()),
            ("0x1", framework_address),
            ("0x10", non_special_short_address),
        ] {
            let parsed = AccountAddress::from_hex(input).unwrap();
            let encoded = parsed.encode();

            assert!(validate_address(input));
            assert_eq!(parsed.as_bytes().len(), 32);
            assert_eq!(encoded.len(), 66);
            assert_eq!(encoded, expected);
        }

        assert_eq!(AccountAddress::from_hex(padded).unwrap(), AccountAddress::from_hex(unpadded).unwrap());
        assert_eq!(AccountAddress::from_hex(reported_padded).unwrap(), AccountAddress::from_hex(reported_unpadded).unwrap());
        assert!(!validate_address("invalid"));
        assert!(!validate_address(&format!("0x{}", "1".repeat(65))));
    }

    #[test]
    fn test_primary_store() {
        let referral = AccountAddress::from_hex("0xc09d385527743bb03ed7847bb9180b5ff2263d38d5a93f1c9b3068f8505f6488").unwrap();
        let wallet = AccountAddress::from_hex("0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef").unwrap();
        let apt = AccountAddress::from_hex("0xa").unwrap();
        let usdc = AccountAddress::from_hex("0xbae207659db88bea0cbead6da0ed00aac12edcdda169e591cd41c94180b46f3b").unwrap();

        assert_eq!(referral.primary_store(&apt).to_string(), "0x9399bcc487e9bd5d941439fbae5ad93aa3b60e706306ee7dc3c4322597a3eaeb");
        assert_eq!(wallet.primary_store(&apt).to_string(), "0x532a89b74cf4a98dc98bcd49adbb3823838344840ccff1d69c5354354cf75ae4");
        assert_eq!(wallet.primary_store(&usdc).to_string(), "0xe221eef6bda1c8c3162e147308acb3d6eb5b6e91851f4e865da50685775d581e");
    }
}
