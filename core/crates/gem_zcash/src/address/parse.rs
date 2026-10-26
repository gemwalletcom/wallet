use primitives::SignerError;
use zcash_address::{ConversionError, TryFromAddress, ZcashAddress};
use zcash_protocol::consensus::NetworkType;

use super::{destination::Destination, transparent::TransparentAddress};

pub(crate) trait ParseAddress: TryFromAddress<Error = SignerError> {
    fn parse(address: &str) -> Result<Self, SignerError> {
        let address = match address.bytes().any(|byte| byte.is_ascii_lowercase()) {
            true => address.to_string(),
            false => address.to_ascii_lowercase(),
        };
        let address = ZcashAddress::try_from_encoded(&address).map_err(|_| SignerError::invalid_input("invalid Zcash address"))?;
        address.convert_if_network(NetworkType::Main).map_err(|error| match error {
            ConversionError::User(error) => error,
            ConversionError::IncorrectNetwork { .. } => SignerError::invalid_input("unsupported Zcash network"),
            ConversionError::Unsupported(_) => SignerError::invalid_input("unsupported Zcash address"),
        })
    }
}

impl<T: TryFromAddress<Error = SignerError>> ParseAddress for T {}

pub fn validate_address(address: &str) -> bool {
    Destination::parse(address).is_ok()
}

pub fn transparent_address(address: &str) -> Result<String, SignerError> {
    Ok(TransparentAddress::parse(address)?.encode())
}

#[cfg(test)]
mod tests {
    use primitives::testkit::zcash_mock::{TEST_ZCASH_TEX_ADDRESS, TEST_ZCASH_TRANSPARENT_ADDRESS};

    use super::*;
    use crate::testkit::{TEST_ZCASH_UNIFIED_ADDRESS, TEST_ZCASH_UNIFIED_SAPLING_ADDRESS};

    #[test]
    fn test_validate_address() {
        assert!(validate_address(TEST_ZCASH_TRANSPARENT_ADDRESS));
        assert!(validate_address(TEST_ZCASH_TEX_ADDRESS));
        assert!(validate_address(TEST_ZCASH_UNIFIED_ADDRESS));
        assert!(!validate_address(TEST_ZCASH_UNIFIED_SAPLING_ADDRESS));
        assert!(!validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"));
    }

    #[test]
    fn test_transparent_address() {
        assert_eq!(transparent_address(TEST_ZCASH_TEX_ADDRESS).unwrap(), TEST_ZCASH_TRANSPARENT_ADDRESS);
        assert_eq!(transparent_address(TEST_ZCASH_TRANSPARENT_ADDRESS).unwrap(), TEST_ZCASH_TRANSPARENT_ADDRESS);
        assert_eq!(transparent_address("t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd").unwrap(), "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd");
        assert_eq!(transparent_address(TEST_ZCASH_UNIFIED_ADDRESS).unwrap_err(), SignerError::invalid_input("unsupported Zcash address"));
    }
}
