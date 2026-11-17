use gem_encoding::{decode_base58, encode_base58};
use primitives::decode_hex;
use primitives::hex::encode_with_0x;
use zeroize::Zeroizing;

use super::{aptos, stellar, sui};
use crate::{SignatureScheme, SignerError};

#[derive(Debug, Clone, Copy)]
pub(super) enum KeyEncoding {
    Hex,
    Base58,
    PrefixedBase58(SignatureScheme),
    PrefixedHex(SignatureScheme),
    StellarStrKey,
    SuiBech32,
}

impl KeyEncoding {
    pub(super) fn decode(self, value: &str) -> Option<Zeroizing<Vec<u8>>> {
        match self {
            Self::Hex => decode_hex(value).ok().map(Zeroizing::new),
            Self::Base58 => decode_base58_keypair(value).map(Zeroizing::new),
            Self::PrefixedBase58(scheme) => decode_prefixed_base58(value, scheme).map(Zeroizing::new),
            Self::PrefixedHex(scheme) => aptos::decode_prefixed_hex(value, scheme).map(Zeroizing::new),
            Self::StellarStrKey => stellar::decode_secret_seed(value),
            Self::SuiBech32 => sui::decode_sui_bech32(value).map(Zeroizing::new),
        }
    }

    pub(super) fn encode(self, bytes: &[u8]) -> Result<String, SignerError> {
        match self {
            Self::Hex => Ok(encode_with_0x(bytes)),
            Self::Base58 => Ok(encode_base58(bytes)),
            Self::PrefixedBase58(scheme) => Ok(format!("{}:{}", scheme.as_ref(), encode_base58(bytes))),
            Self::PrefixedHex(scheme) => Ok(aptos::encode_prefixed_hex(bytes, scheme)),
            Self::StellarStrKey => stellar::encode_secret_seed(bytes),
            Self::SuiBech32 => sui::encode_sui_bech32(bytes),
        }
    }
}

fn decode_base58_keypair(value: &str) -> Option<Vec<u8>> {
    let decoded = Zeroizing::new(decode_base58(value).ok()?);
    let keypair = decoded.as_slice().try_into().ok()?;
    ed25519_dalek::SigningKey::from_keypair_bytes(keypair).ok()?;
    Some(decoded[..32].to_vec())
}

fn decode_prefixed_base58(value: &str, scheme: SignatureScheme) -> Option<Vec<u8>> {
    decode_base58_keypair(value.strip_prefix(scheme.as_ref())?.strip_prefix(':')?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_rejects_unexpected_length() {
        let long_key = encode_base58(vec![1u8; 96]);
        assert!(decode_base58_keypair(&long_key).is_none());
    }
}
