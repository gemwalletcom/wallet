use gem_hash::sha2::sha256;

use crate::{EncodingError, EncodingType};

const CHECKSUM_LENGTH: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum Base58Alphabet {
    Bitcoin,
    Ripple,
}

impl Base58Alphabet {
    fn alphabet(self) -> &'static bs58::Alphabet {
        match self {
            Self::Bitcoin => bs58::Alphabet::BITCOIN,
            Self::Ripple => bs58::Alphabet::RIPPLE,
        }
    }
}

pub fn encode_base58(bytes: impl AsRef<[u8]>) -> String {
    bs58::encode(bytes).into_string()
}

pub fn decode_base58(value: impl AsRef<[u8]>) -> Result<Vec<u8>, EncodingError> {
    bs58::decode(value).into_vec().map_err(|error| invalid(error.to_string()))
}

pub fn decode_base58_into(value: impl AsRef<[u8]>, output: &mut [u8]) -> Result<usize, EncodingError> {
    bs58::decode(value).onto(output).map_err(|error| invalid(error.to_string()))
}

pub fn encode_base58_check(payload: &[u8], alphabet: Base58Alphabet) -> String {
    let mut bytes = Vec::with_capacity(payload.len() + CHECKSUM_LENGTH);
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&checksum(payload));
    bs58::encode(bytes).with_alphabet(alphabet.alphabet()).into_string()
}

pub fn decode_base58_check(value: &str, alphabet: Base58Alphabet) -> Result<Vec<u8>, EncodingError> {
    let mut bytes = bs58::decode(value).with_alphabet(alphabet.alphabet()).into_vec().map_err(|error| invalid(error.to_string()))?;
    let length = bytes.len().checked_sub(CHECKSUM_LENGTH).ok_or_else(|| invalid("missing checksum"))?;
    if bytes[length..] != checksum(&bytes[..length]) {
        return Err(invalid("invalid checksum"));
    }
    bytes.truncate(length);
    Ok(bytes)
}

fn checksum(payload: &[u8]) -> [u8; CHECKSUM_LENGTH] {
    let hash = sha256(&sha256(payload));
    [hash[0], hash[1], hash[2], hash[3]]
}

fn invalid(message: impl Into<String>) -> EncodingError {
    EncodingError::Invalid(EncodingType::Base58, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode_hex;

    #[test]
    fn test_base58() {
        let bytes = decode_hex("00010966776006953d5567439e5e39f86a0d273bee").unwrap();
        let encoded = "1qb3y62fmEEVTPySXPQ77WXok6H";

        assert_eq!(encode_base58(&bytes), encoded);
        assert_eq!(decode_base58(encoded).unwrap(), bytes);
        assert_eq!(encode_base58([]), "");
        assert_eq!(decode_base58("").unwrap(), Vec::<u8>::new());
        assert!(decode_base58("0OIl").is_err());

        let mut output = [0u8; 21];
        assert_eq!(decode_base58_into(encoded, &mut output).unwrap(), 21);
        assert_eq!(output.as_slice(), bytes);
        assert!(decode_base58_into(encoded, &mut [0u8; 20]).is_err());
    }

    #[test]
    fn test_base58_check() {
        let payload = decode_hex("00f54a5851e9372b87810a8e60cdd2e7cfd80b6e31").unwrap();
        let bitcoin = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs";
        let ripple = "rPMyc2c8J2SqAAJqj2AXBNi8L1ZfRkX7w1";

        assert_eq!(encode_base58_check(&payload, Base58Alphabet::Bitcoin), bitcoin);
        assert_eq!(decode_base58_check(bitcoin, Base58Alphabet::Bitcoin).unwrap(), payload);
        assert_eq!(encode_base58_check(&payload, Base58Alphabet::Ripple), ripple);
        assert_eq!(decode_base58_check(ripple, Base58Alphabet::Ripple).unwrap(), payload);
        assert_eq!(encode_base58_check(&[], Base58Alphabet::Bitcoin), "3QJmnh");
        assert_eq!(decode_base58_check("3QJmnh", Base58Alphabet::Bitcoin).unwrap(), Vec::<u8>::new());

        assert!(decode_base58_check("1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAt", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check(bitcoin, Base58Alphabet::Ripple).is_err());
        assert!(decode_base58_check("111", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check("", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check("0OIl", Base58Alphabet::Bitcoin).is_err());
    }
}
