use std::iter;

use gem_hash::sha2::sha256;
use zeroize::Zeroizing;

use crate::{EncodingError, EncodingType};

const CHECKSUM_LENGTH: usize = 4;
const BASE: u64 = 58;
const DIGITS_PER_LIMB: usize = 5;
const LIMB_BASE: u64 = BASE.pow(DIGITS_PER_LIMB as u32);
const LIMB_BITS: usize = LIMB_BASE.ilog2() as usize;
const BYTES_PER_STEP: usize = 4;
const INVALID: u8 = u8::MAX;

static BITCOIN: Alphabet = Alphabet::new(b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
static RIPPLE: Alphabet = Alphabet::new(b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz");

#[derive(Debug, Clone, Copy)]
pub enum Base58Alphabet {
    Bitcoin,
    Ripple,
}

impl Base58Alphabet {
    fn alphabet(self) -> &'static Alphabet {
        match self {
            Self::Bitcoin => &BITCOIN,
            Self::Ripple => &RIPPLE,
        }
    }
}

struct Alphabet {
    symbols: &'static [u8; 58],
    values: [u8; 256],
}

impl Alphabet {
    const fn new(symbols: &'static [u8; 58]) -> Self {
        let mut values = [INVALID; 256];
        let mut index = 0;
        while index < symbols.len() {
            values[symbols[index] as usize] = index as u8;
            index += 1;
        }
        Self { symbols, values }
    }

    fn encode(&self, bytes: &[u8]) -> String {
        let zeros = bytes.iter().take_while(|byte| **byte == 0).count();
        let value = &bytes[zeros..];
        let mut limbs = Zeroizing::new(Vec::<u32>::with_capacity((value.len() * 8).div_ceil(LIMB_BITS)));
        for chunk in value.rchunks(BYTES_PER_STEP).rev() {
            let shift = 8 * chunk.len();
            let mut carry = chunk.iter().fold(0u64, |value, byte| (value << 8) | u64::from(*byte));
            for limb in limbs.iter_mut() {
                let value = (u64::from(*limb) << shift) + carry;
                *limb = (value % LIMB_BASE) as u32;
                carry = value / LIMB_BASE;
            }
            while carry > 0 {
                limbs.push((carry % LIMB_BASE) as u32);
                carry /= LIMB_BASE;
            }
        }
        let mut text = String::with_capacity(zeros + limbs.len() * DIGITS_PER_LIMB);
        text.extend(iter::repeat_n(char::from(self.symbols[0]), zeros));
        for (position, limb) in limbs.iter().rev().enumerate() {
            let mut digits = [0u8; DIGITS_PER_LIMB];
            let mut rest = u64::from(*limb);
            for digit in digits.iter_mut().rev() {
                *digit = (rest % BASE) as u8;
                rest /= BASE;
            }
            let start = if position == 0 { digits.iter().take_while(|digit| **digit == 0).count() } else { 0 };
            text.extend(digits[start..].iter().map(|digit| char::from(self.symbols[usize::from(*digit)])));
        }
        text
    }

    fn decode(&self, text: &[u8]) -> Result<Vec<u8>, EncodingError> {
        if let Some(index) = text.iter().position(|symbol| self.values[usize::from(*symbol)] == INVALID) {
            return Err(invalid(format!("invalid symbol at byte {index}")));
        }
        let zeros = text.iter().take_while(|symbol| **symbol == self.symbols[0]).count();
        let digits = &text[zeros..];
        let mut limbs = Zeroizing::new(Vec::<u32>::with_capacity(digits.len().div_ceil(DIGITS_PER_LIMB)));
        for chunk in digits.rchunks(DIGITS_PER_LIMB).rev() {
            let (mut carry, multiplier) = chunk.iter().fold((0u64, 1u64), |(value, multiplier), symbol| (value * BASE + u64::from(self.values[usize::from(*symbol)]), multiplier * BASE));
            for limb in limbs.iter_mut() {
                let value = u64::from(*limb) * multiplier + carry;
                *limb = value as u32;
                carry = value >> 32;
            }
            while carry > 0 {
                limbs.push(carry as u32);
                carry >>= 32;
            }
        }
        let mut bytes = Vec::with_capacity(zeros + limbs.len() * size_of::<u32>());
        bytes.resize(zeros, 0);
        if let Some((last, rest)) = limbs.split_last() {
            bytes.extend(last.to_be_bytes().into_iter().skip_while(|byte| *byte == 0));
            for limb in rest.iter().rev() {
                bytes.extend_from_slice(&limb.to_be_bytes());
            }
        }
        Ok(bytes)
    }
}

pub fn encode_base58(bytes: impl AsRef<[u8]>) -> String {
    BITCOIN.encode(bytes.as_ref())
}

pub fn decode_base58(value: impl AsRef<[u8]>) -> Result<Vec<u8>, EncodingError> {
    BITCOIN.decode(value.as_ref())
}

pub fn decode_base58_into(value: impl AsRef<[u8]>, output: &mut [u8]) -> Result<usize, EncodingError> {
    let bytes = Zeroizing::new(BITCOIN.decode(value.as_ref())?);
    output.get_mut(..bytes.len()).ok_or_else(|| invalid("output buffer is too small"))?.copy_from_slice(&bytes);
    Ok(bytes.len())
}

pub fn encode_base58_check(payload: &[u8], alphabet: Base58Alphabet) -> String {
    let mut bytes = Zeroizing::new(Vec::with_capacity(payload.len() + CHECKSUM_LENGTH));
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&checksum(payload));
    alphabet.alphabet().encode(&bytes)
}

pub fn decode_base58_check(value: &str, alphabet: Base58Alphabet) -> Result<Vec<u8>, EncodingError> {
    let mut bytes = alphabet.alphabet().decode(value.as_bytes())?;
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

    const DIGITS_OF_PI: &str = "03243f6a8885a308d313198a2e03707344a4093822299f31d0082efa98ec4e6c89452821e638d01377be5466cf34e90c6cc0ac29b7c97c50dd3f84d5b5b54709179216d5d98979fb1bd1310ba698dfb5ac2ffd72dbd01adfb7b8e1afed6a267e96ba7c9045f12c7f9924a19947b3916cf70801f2e2858efc16636920d871574e";

    const CASES: [(&str, &str); 20] = [
        ("", ""),
        ("61", "2g"),
        ("626262", "a3gV"),
        ("636363", "aPEr"),
        ("572e4794", "3EFU7m"),
        ("10c8511e", "Rt5zm"),
        ("516b6fcd0f", "ABnLTmg"),
        ("bf4f89001e670274dd", "3SEo3LWLoPntC"),
        ("ecac89cad93923c02321", "EJDM8drfXA6uyA"),
        ("00000000000000000000", "1111111111"),
        ("ffffffffffffffffffff", "FPBt6CHo3fovdL"),
        ("ffffffffffffffffffffffffff", "NKioeUVktgzXLJ1B3t"),
        ("ffffffffffffffffffffffffffffffff", "YcVfxkQb6JRzqk5kF2tNLv"),
        ("73696d706c792061206c6f6e6720737472696e67", "2cFupjhnEsSn59qHXstmK2ffpLv2"),
        ("00eb15231dfceb60925886b67d065299925915aeb172c06647", "1NS17iag9jJgTHD1VXjvLCEnZuQ3rJDE9L"),
        ("003c176e659bea0f29a3e9bf7880c112b1b31b4dc826268187", "16UjcYNBG9GTK4uq2f7yYEbuifqCzoLMGS"),
        ("0000000000000000000000000000000000000000000000000000000000000000", "11111111111111111111111111111111"),
        ("801184cd2cdd640ca42cfc3a091c51d549b2f016d454b2774019c2b2d2e08529fd206ec97e", "5Hx15HFGyep2CfPxsJKe2fXJsCVn5DEiyoeGGF6JZjGbTRnqfiD"),
        (
            DIGITS_OF_PI,
            "KeThPkHTv5nsa4576Z47NqEtuSfUcKwv7YeueZ8dquGTDeBpimjGEZ1a7k1FCz8m8FEBcoJZjP5Aui6eKfPjdmGooHKtEPRbVotw6mRxNU3WbLtAH41mea9g8AB9Qe1DAFDReBWa67ZEP6ApWGhw9Dfr2vVXkLXEWj6W8HFApw4DKK",
        ),
        ("00010966776006953d5567439e5e39f86a0d273bee", "1qb3y62fmEEVTPySXPQ77WXok6H"),
    ];

    const CHECK_CASES: [(&str, &str); 9] = [
        ("", "3QJmnh"),
        ("31", "6bdbJ1U"),
        ("39", "7VsrQCP"),
        ("2d31", "PWEu9GGN"),
        ("3131", "RVnPfpC2"),
        ("31323334353938373630", "K5zqBMZZTzUbAZQgrt4"),
        ("009b4154bbf203e4130c4b86259318a49875dd0456", "1F9v11cupBVMpz3CrVfCppv9Rw2xEtU1c6"),
        ("5325b1e23b5b24f347ed19de61238af14bc471caa1a77aa55db2a7af7daa93aa", "dctKSXBbv2My3TGGUgTFjkxu1A9JM3Sscd5FydY4dkxnfwA7q"),
        (
            DIGITS_OF_PI,
            "371hJQw3jVfFQtQfQ1NnUFV4Z3i166yKJe3yyPAvJziEfUenJBD8SM6xGFop9cfCDCn4j9HcT9fS73jgGp8XZzYKmSxjxLcxfgETzg4BcDHLgHSynSFDGR5wJ58NkZSv2mVxvqVwG8hqxNFXrWms66ppx45yAjc7dYuBXqCPZ2GatCMmrhuX",
        ),
    ];

    #[test]
    fn test_base58() {
        for (hex, encoded) in CASES {
            let bytes = decode_hex(hex).unwrap();
            assert_eq!(encode_base58(&bytes), encoded, "{hex}");
            assert_eq!(decode_base58(encoded).unwrap(), bytes, "{encoded}");

            let mut output = [7u8; 128];
            assert_eq!(decode_base58_into(encoded, &mut output).unwrap(), bytes.len());
            assert_eq!(&output[..bytes.len()], bytes.as_slice());
            assert_eq!(output[bytes.len()..], [7u8; 128][bytes.len()..]);
        }
    }

    #[test]
    fn test_base58_invalid() {
        for invalid in ["0", "O", "I", "l", "123456789abcd!efghij", "2g ", " 2g", "2g\n", "2\u{e9}g", "\u{0}"] {
            assert!(decode_base58(invalid).is_err(), "{invalid:?}");
        }
        assert_eq!(decode_base58("123456789abcd!efghij").unwrap_err().to_string(), invalid("invalid symbol at byte 13").to_string());
        assert!(decode_base58_into("a3gV", &mut [0u8; 2]).is_err());
        assert!(decode_base58_into("1qb3y62fmEEVTPySXPQ77WXok6H", &mut [0u8; 20]).is_err());
    }

    #[test]
    fn test_base58_check() {
        for (hex, encoded) in CHECK_CASES {
            let bytes = decode_hex(hex).unwrap();
            assert_eq!(encode_base58_check(&bytes, Base58Alphabet::Bitcoin), encoded, "{hex}");
            assert_eq!(decode_base58_check(encoded, Base58Alphabet::Bitcoin).unwrap(), bytes, "{encoded}");
        }

        let payload = decode_hex("00f54a5851e9372b87810a8e60cdd2e7cfd80b6e31").unwrap();
        let bitcoin = "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs";
        let ripple = "rPMyc2c8J2SqAAJqj2AXBNi8L1ZfRkX7w1";

        assert_eq!(encode_base58_check(&payload, Base58Alphabet::Bitcoin), bitcoin);
        assert_eq!(encode_base58_check(&payload, Base58Alphabet::Ripple), ripple);
        assert_eq!(decode_base58_check(ripple, Base58Alphabet::Ripple).unwrap(), payload);

        assert!(decode_base58_check("1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAt", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check(bitcoin, Base58Alphabet::Ripple).is_err());
        assert!(decode_base58_check("111", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check("", Base58Alphabet::Bitcoin).is_err());
        assert!(decode_base58_check("0OIl", Base58Alphabet::Bitcoin).is_err());
    }
}
