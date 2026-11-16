use zeroize::Zeroizing;

use crate::base_n::Alphabet;
use crate::{EncodingError, EncodingType, crc16_xmodem};

static STANDARD: Alphabet<5, 8> = Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");

const STRKEY_LENGTH: usize = 56;
const STRKEY_PAYLOAD_LENGTH: usize = 32;
const STRKEY_CHECKSUM_OFFSET: usize = STRKEY_PAYLOAD_LENGTH + 1;

pub fn encode_base32(bytes: &[u8]) -> String {
    STANDARD.encode(bytes, false)
}

pub fn decode_base32(value: &[u8]) -> Result<Vec<u8>, EncodingError> {
    STANDARD.decode(value, EncodingType::Base32)
}

pub fn encode_strkey(version: u8, payload: &[u8; STRKEY_PAYLOAD_LENGTH]) -> String {
    let body = [&[version], payload.as_slice()].concat();
    let checksum = crc16_xmodem(&body).to_le_bytes();
    encode_base32(&[body.as_slice(), &checksum].concat())
}

pub fn decode_strkey(value: &str, version: u8) -> Result<Zeroizing<Vec<u8>>, EncodingError> {
    if value.len() != STRKEY_LENGTH {
        return Err(invalid("invalid strkey length"));
    }
    let bytes = Zeroizing::new(decode_base32(value.as_bytes())?);
    if bytes[0] != version {
        return Err(invalid("invalid strkey version"));
    }
    if bytes[STRKEY_CHECKSUM_OFFSET..] != crc16_xmodem(&bytes[..STRKEY_CHECKSUM_OFFSET]).to_le_bytes() {
        return Err(invalid("invalid strkey checksum"));
    }
    Ok(Zeroizing::new(bytes[1..STRKEY_CHECKSUM_OFFSET].to_vec()))
}

fn invalid(message: &str) -> EncodingError {
    EncodingError::Invalid(EncodingType::Base32, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_hex;

    #[test]
    fn test_base32_rfc_4648_vectors() {
        for (plain, encoded) in [("", ""), ("f", "MY"), ("fo", "MZXQ"), ("foo", "MZXW6"), ("foob", "MZXW6YQ"), ("fooba", "MZXW6YTB"), ("foobar", "MZXW6YTBOI")] {
            assert_eq!(encode_base32(plain.as_bytes()), encoded);
            assert_eq!(decode_base32(encoded.as_bytes()).unwrap(), plain.as_bytes(), "{encoded}");
        }
    }

    #[test]
    fn test_base32_round_trip_every_length() {
        for length in 0..=64usize {
            let bytes: Vec<u8> = (0..length).map(|index| (index * 37 + 251) as u8).collect();
            assert_eq!(decode_base32(encode_base32(&bytes).as_bytes()).unwrap(), bytes);
        }
    }

    #[test]
    fn test_decode_base32_rejects_non_canonical_input() {
        for value in ["M", "MZX", "MZXW6Y", "MZ", "MZXR", "MY======", "my", "MZXW6=", "MZXW1", "MZXW 6", "MZXW6\n", "\u{e9}Y"] {
            assert!(decode_base32(value.as_bytes()).is_err(), "{value:?}");
        }
    }

    #[test]
    fn test_strkey() {
        let address = "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI";
        let payload = decode_strkey(address, 0x30).unwrap();

        assert_eq!(encode_hex(payload.as_slice()), "09a966bcaacc103e38896baae3f8c2f06c21fd47dd4f864ff0d33f9819df5ca2");
        assert_eq!(encode_strkey(0x30, payload.as_slice().try_into().unwrap()), address);

        assert!(decode_strkey(address, 0x90).is_err());
        assert!(decode_strkey("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYZ", 0x30).is_err());
        assert!(decode_strkey(&address[..55], 0x30).is_err());
        assert!(decode_strkey(&format!("{address}A"), 0x30).is_err());
        assert!(decode_strkey(&address.to_lowercase(), 0x30).is_err());
        assert!(decode_strkey("", 0x30).is_err());
    }
}
