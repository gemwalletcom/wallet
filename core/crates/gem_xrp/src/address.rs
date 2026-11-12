use std::fmt;

use gem_encoding::{Base58Alphabet, decode_base58_check, encode_base58_check};
use primitives::Address;
#[cfg(feature = "signer")]
use primitives::SignerError;

const CLASSIC_PUBLIC_KEY_HASH_LENGTH: usize = 20;
const CLASSIC_ADDRESS_LENGTH: usize = CLASSIC_PUBLIC_KEY_HASH_LENGTH + 1;
const CLASSIC_PREFIX: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XrpAddress([u8; CLASSIC_PUBLIC_KEY_HASH_LENGTH]);

impl Address for XrpAddress {
    fn try_parse(value: &str) -> Option<Self> {
        let decoded = decode_base58_check(value, Base58Alphabet::Ripple).ok()?;
        let [CLASSIC_PREFIX, public_key_hash @ ..] = decoded.as_slice() else {
            return None;
        };

        Some(Self(public_key_hash.try_into().ok()?))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(CLASSIC_ADDRESS_LENGTH);
        raw.push(CLASSIC_PREFIX);
        raw.extend_from_slice(&self.0);
        encode_base58_check(&raw, Base58Alphabet::Ripple)
    }
}

impl XrpAddress {
    pub fn from_public_key_hash(public_key_hash: [u8; CLASSIC_PUBLIC_KEY_HASH_LENGTH]) -> Self {
        Self(public_key_hash)
    }

    #[cfg(feature = "signer")]
    pub(crate) fn parse(value: &str) -> Result<Self, SignerError> {
        Self::try_parse(value).ok_or_else(|| SignerError::invalid_input("invalid XRP classic address"))
    }

    #[cfg(feature = "signer")]
    pub(crate) fn as_bytes(&self) -> &[u8; CLASSIC_PUBLIC_KEY_HASH_LENGTH] {
        &self.0
    }
}

pub fn validate_address(address: &str) -> bool {
    XrpAddress::is_valid(address)
}

impl fmt::Display for XrpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;

    const CLASSIC_ADDRESS: &str = "rnBFvgZphmN39GWzUJeUitaP22Fr9be75H";

    #[test]
    fn test_parse_addresses() {
        let parsed = XrpAddress::from_str(CLASSIC_ADDRESS).unwrap();

        assert_eq!(hex::encode(parsed.as_bytes()), "2decab42ca805119a9ba2ff305c9afa12f0b86a1");
        assert_eq!(parsed.to_string(), CLASSIC_ADDRESS);
        assert!(validate_address(CLASSIC_ADDRESS));
        assert!(XrpAddress::from_str("invalid").is_err());
        assert!(!validate_address("rnBFvgZphmN39GWzUJeUitaP22Fr9be75J"));
    }

    #[test]
    fn test_xrp_address_from_public_key_hash() {
        let public_key_hash = hex::decode("2decab42ca805119a9ba2ff305c9afa12f0b86a1").unwrap().try_into().unwrap();

        assert_eq!(XrpAddress::from_public_key_hash(public_key_hash).to_string(), CLASSIC_ADDRESS);
    }
}
