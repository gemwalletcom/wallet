pub mod serializer;

use std::fmt;

#[cfg(feature = "signer")]
use gem_hash::keccak::keccak256;
#[cfg(feature = "signer")]
use primitives::SignerError;
use primitives::{Address as AddressTrait, AddressError, decode_hex};
#[cfg(feature = "signer")]
use signer::secp256k1_uncompressed_public_key;

const ADDRESS_PREFIX: u8 = 0x41;
const ADDRESS_LEN: usize = 20;
const PREFIXED_ADDRESS_LEN: usize = ADDRESS_LEN + 1;
const ABI_ADDRESS_PARAMETER_HEX_LEN: usize = 64;
#[cfg(feature = "signer")]
const SECP256K1_UNCOMPRESSED_PUBLIC_KEY_PREFIX: u8 = 0x04;
#[cfg(feature = "signer")]
const SECP256K1_UNCOMPRESSED_PUBLIC_KEY_LEN: usize = 65;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TronAddress([u8; PREFIXED_ADDRESS_LEN]);

impl TronAddress {
    pub fn from_hex(hex_value: &str) -> Option<Self> {
        let bytes = decode_hex(hex_value).ok()?;
        match bytes.len() {
            PREFIXED_ADDRESS_LEN if bytes.first() == Some(&ADDRESS_PREFIX) => Some(Self(bytes.try_into().ok()?)),
            ADDRESS_LEN => bytes.try_into().ok().map(|account_id: [u8; ADDRESS_LEN]| Self::from(account_id)),
            _ => None,
        }
    }

    pub fn from_hex_or_base58(value: &str) -> Option<Self> {
        // v3-compatible raw transaction parsing prefers base58 when both formats are technically valid.
        Self::try_parse(value).or_else(|| Self::from_hex(value))
    }

    pub fn abi_address_parameter(&self) -> String {
        format!("{:0>width$}", hex::encode(self.account_id()), width = ABI_ADDRESS_PARAMETER_HEX_LEN)
    }

    pub fn parse(address: &str) -> Result<Self, AddressError> {
        Self::try_parse(address).ok_or_else(|| AddressError::new(format!("invalid Tron address: {address}")))
    }

    pub fn account_id(&self) -> &[u8] {
        &self.0[1..]
    }

    #[cfg(feature = "signer")]
    pub(crate) fn from_private_key(private_key: &[u8]) -> Result<Self, SignerError> {
        let public_key = secp256k1_uncompressed_public_key(private_key)?;
        if public_key.len() != SECP256K1_UNCOMPRESSED_PUBLIC_KEY_LEN || public_key.first() != Some(&SECP256K1_UNCOMPRESSED_PUBLIC_KEY_PREFIX) {
            return SignerError::invalid_input_err("Invalid Secp256k1 public key");
        }

        let hash = keccak256(&public_key[1..]);
        let account_id: [u8; ADDRESS_LEN] = hash[hash.len() - ADDRESS_LEN..]
            .try_into()
            .map_err(|_| SignerError::invalid_input("invalid Tron account id length"))?;
        Ok(Self::from(account_id))
    }
}

impl AddressTrait for TronAddress {
    fn try_parse(address: &str) -> Option<Self> {
        let decoded = bs58::decode(address).with_check(None).into_vec().ok()?;
        let payload = match decoded.as_slice() {
            [ADDRESS_PREFIX, payload @ ..] => payload,
            // v3 accepts 20-byte base58check payloads and normalizes them with the Tron prefix.
            payload => payload,
        };

        let account_id: [u8; ADDRESS_LEN] = payload.try_into().ok()?;
        Some(Self::from(account_id))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        bs58::encode(self.0).with_check().into_string()
    }
}

pub fn validate_address(address: &str) -> bool {
    TronAddress::is_valid(address)
}

impl From<[u8; ADDRESS_LEN]> for TronAddress {
    fn from(address: [u8; ADDRESS_LEN]) -> Self {
        let mut bytes = [0u8; PREFIXED_ADDRESS_LEN];
        bytes[0] = ADDRESS_PREFIX;
        bytes[1..].copy_from_slice(&address);
        Self(bytes)
    }
}

impl fmt::Display for TronAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(
            TronAddress::from_hex("4159f3440fd40722f716144e4490a4de162d3b3fcb").unwrap().encode(),
            "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH".to_string()
        );
        assert_eq!(
            TronAddress::from_hex("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().encode(),
            "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1".to_string()
        );
        assert_eq!(
            TronAddress::from_hex("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().to_string(),
            "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1"
        );
        assert_eq!(
            TronAddress::from_hex("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().to_string(),
            "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1"
        );
        assert_eq!(TronAddress::from_hex("42357a7401a0f0c2d4a44a1881a0c622f15d986291"), None);
    }

    #[test]
    fn test_to_addr_from_base58() {
        let expected: [u8; ADDRESS_LEN] = hex::decode("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().try_into().unwrap();
        assert_eq!(TronAddress::parse("TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1").unwrap().account_id(), expected);
        assert_eq!(TronAddress::parse("invalid").unwrap_err().to_string(), "invalid Tron address: invalid");
    }

    #[test]
    fn test_abi_address_parameter() {
        assert_eq!(
            TronAddress::parse("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb").unwrap().abi_address_parameter(),
            "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f"
        );
        assert!(TronAddress::parse("invalid").is_err());
    }

    #[test]
    fn test_try_parse_normalizes_prefixed_and_unprefixed_payloads() {
        let prefixed = "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1";
        let payload = hex::decode("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();
        let unprefixed = bs58::encode(&payload).with_check().into_string();
        let expected = hex::decode("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();

        assert!(validate_address(prefixed));
        assert!(validate_address(&unprefixed));
        assert_eq!(TronAddress::try_parse(prefixed).unwrap().as_bytes(), expected);
        assert_eq!(TronAddress::try_parse(&unprefixed).unwrap().as_bytes(), expected);
    }

    #[test]
    fn test_from_hex_or_base58() {
        let expected = hex::decode("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();
        let chainflip_expected = hex::decode("412523ae929fecd9d665f472f59b99a8ce6b179510").unwrap();

        assert_eq!(TronAddress::from_hex_or_base58("TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1").unwrap().as_bytes(), expected);
        assert_eq!(TronAddress::from_hex_or_base58("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().as_bytes(), expected);
        assert_eq!(TronAddress::from_hex_or_base58("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap().as_bytes(), expected);
        assert_eq!(
            TronAddress::from_hex_or_base58("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi").unwrap().as_bytes(),
            chainflip_expected
        );
        assert_eq!(
            TronAddress::from_hex_or_base58("412523ae929fecd9d665f472f59b99a8ce6b179510").unwrap().as_bytes(),
            chainflip_expected
        );
        assert_eq!(
            TronAddress::from_hex_or_base58("2523ae929fecd9d665f472f59b99a8ce6b179510").unwrap().as_bytes(),
            chainflip_expected
        );
        assert_eq!(TronAddress::from_hex_or_base58("invalid"), None);
    }

    #[cfg(feature = "signer")]
    #[test]
    fn test_from_private_key() {
        let private_key = hex::decode("2d8f68944bdbfbc0769542fba8fc2d2a3de67393334471624364c7006da2aa54").unwrap();
        assert_eq!(TronAddress::from_private_key(&private_key).unwrap().encode(), "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC");
        assert!(TronAddress::from_private_key(&[0u8; 16]).is_err());
    }

    #[test]
    fn test_try_parse_rejects_wrong_prefix() {
        let mut decoded = hex::decode("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();
        decoded[0] = 0x42;
        let address = bs58::encode(decoded).with_check().into_string();

        assert!(TronAddress::try_parse(&address).is_none());
        assert!(!validate_address(&address));
    }
}
