use gem_encoding::{decode_base58, encode_base58};
use primitives::decode_hex;
use primitives::hex::encode_with_0x;
use zeroize::Zeroizing;

use super::{aptos, stellar, sui};
use crate::SignerError;

const NEAR_ED25519_PREFIX: &str = "ed25519:";

#[derive(Debug, Clone, Copy)]
pub(super) enum KeyEncoding {
    Hex,
    Base58,
    PrefixedBase58,
    PrefixedHex,
    StellarStrKey,
    SuiBech32,
}

impl KeyEncoding {
    pub(super) fn decode(self, value: &str) -> Option<Zeroizing<Vec<u8>>> {
        match self {
            Self::Hex => decode_hex(value).ok().map(Zeroizing::new),
            Self::Base58 => decode_base58_keypair(value),
            Self::PrefixedBase58 => decode_base58_keypair(value.strip_prefix(NEAR_ED25519_PREFIX)?),
            Self::PrefixedHex => aptos::decode(value),
            Self::StellarStrKey => stellar::decode(value),
            Self::SuiBech32 => sui::decode(value),
        }
    }

    pub(super) fn encode(self, bytes: &[u8]) -> Result<String, SignerError> {
        match self {
            Self::Hex => Ok(encode_with_0x(bytes)),
            Self::Base58 => Ok(encode_base58(bytes)),
            Self::PrefixedBase58 => Ok(format!("{NEAR_ED25519_PREFIX}{}", encode_base58(bytes))),
            Self::PrefixedHex => Ok(aptos::encode(bytes)),
            Self::StellarStrKey => stellar::encode(bytes),
            Self::SuiBech32 => sui::encode(bytes),
        }
    }
}

fn decode_base58_keypair(value: &str) -> Option<Zeroizing<Vec<u8>>> {
    let decoded = Zeroizing::new(decode_base58(value).ok()?);
    let keypair = decoded.as_slice().try_into().ok()?;
    ed25519_dalek::SigningKey::from_keypair_bytes(keypair).ok()?;
    Some(Zeroizing::new(decoded[..32].to_vec()))
}
