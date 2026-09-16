use crate::{Result, SolanaError};

const MAX_ENCODING_LENGTH: usize = 3;

pub(crate) fn encode_length_to_compact_u16_bytes(len: usize) -> Result<Vec<u8>> {
    let mut remaining = u16::try_from(len).map_err(|_| SolanaError::invalid_input(format!("Length {len} exceeds the compact-u16 range")))?;
    let mut bytes = Vec::with_capacity(MAX_ENCODING_LENGTH);
    loop {
        let mut byte = (remaining & 0x7f) as u8;
        remaining >>= 7;
        if remaining == 0 {
            bytes.push(byte);
            return Ok(bytes);
        }
        byte |= 0x80;
        bytes.push(byte);
    }
}

pub(crate) fn decode_compact_u16_len(bytes: &[u8]) -> Result<(usize, usize)> {
    let mut len = 0_usize;
    for (position, byte) in bytes.iter().copied().take(MAX_ENCODING_LENGTH).enumerate() {
        if byte == 0 && position > 0 {
            return Err(SolanaError::invalid_input("Non-canonical compact-u16 length"));
        }
        len |= (byte as usize & 0x7f) << (position * 7);
        if byte & 0x80 == 0 {
            if len > u16::MAX as usize {
                return Err(SolanaError::invalid_input("Compact-u16 length exceeds u16::MAX"));
            }
            return Ok((len, position + 1));
        }
    }
    Err(SolanaError::invalid_input("Compact-u16 length is truncated or longer than 3 bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_u16_roundtrip_and_boundaries() {
        for len in [0, 1, 127, 128, 16_383, 16_384, u16::MAX as usize] {
            let bytes = encode_length_to_compact_u16_bytes(len).unwrap();
            assert_eq!(decode_compact_u16_len(&bytes).unwrap(), (len, bytes.len()));
        }
        assert_eq!(encode_length_to_compact_u16_bytes(128).unwrap(), [0x80, 0x01]);
        assert_eq!(decode_compact_u16_len(&[0xff, 0xff, 0x03]).unwrap(), (u16::MAX as usize, 3));
        assert!(encode_length_to_compact_u16_bytes(u16::MAX as usize + 1).is_err());
        assert!(decode_compact_u16_len(&[0xff, 0xff, 0x7f]).is_err());
        assert!(decode_compact_u16_len(&[0xff, 0xff, 0xff]).is_err());
        assert!(decode_compact_u16_len(&[0x80]).is_err());
        assert!(decode_compact_u16_len(&[]).is_err());
    }

    #[test]
    fn test_decode_compact_u16_len_rejects_non_canonical_encoding() {
        assert!(decode_compact_u16_len(&[0x80, 0x00]).is_err());
        assert!(decode_compact_u16_len(&[0x81, 0x00]).is_err());
        assert!(decode_compact_u16_len(&[0xff, 0x80, 0x00]).is_err());
    }
}
