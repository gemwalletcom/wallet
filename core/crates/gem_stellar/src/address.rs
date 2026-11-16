use gem_encoding::{decode_strkey, encode_strkey};
use primitives::{Address, SignerError};
use signer::Base32Address;
use std::fmt;

const ED25519_PUBLIC_KEY_VERSION: u8 = 0x30;

#[derive(Clone)]
pub struct StellarAddress {
    pub(crate) base32: Base32Address,
}

impl Address for StellarAddress {
    fn try_parse(address: &str) -> Option<Self> {
        let payload = decode_strkey(address, ED25519_PUBLIC_KEY_VERSION).ok()?;
        Base32Address::from_slice(&payload).ok().map(|base32| Self { base32 })
    }

    fn as_bytes(&self) -> &[u8] {
        self.base32.payload()
    }

    fn encode(&self) -> String {
        encode_strkey(ED25519_PUBLIC_KEY_VERSION, self.base32.payload())
    }
}

pub fn validate_address(address: &str) -> bool {
    StellarAddress::is_valid(address)
}

impl StellarAddress {
    pub fn from_public_key(public_key_bytes: &[u8]) -> Result<Self, SignerError> {
        Ok(Self {
            base32: Base32Address::from_slice(public_key_bytes)?,
        })
    }
}

impl fmt::Display for StellarAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ADDRESS: &str = "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI";

    #[test]
    fn test_stellar_address() {
        let parsed = StellarAddress::from_str(VALID_ADDRESS).unwrap();

        assert!(validate_address(VALID_ADDRESS));
        assert_eq!(parsed.to_string(), VALID_ADDRESS);
        assert_eq!(parsed.as_bytes().len(), 32);

        assert!(!validate_address(""));
        assert!(!validate_address("invalid"));
        assert!(!validate_address("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYZ"));
    }
}
