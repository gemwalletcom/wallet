const STRING_OFFSET: u8 = 0x80;
const LIST_OFFSET: u8 = 0xc0;
const SHORT_PAYLOAD_LIMIT: usize = 55;
const MAX_HEADER_LEN: usize = 9;

pub fn encode_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    if let [byte] = bytes
        && *byte < STRING_OFFSET
    {
        out.push(*byte);
        return;
    }
    out.reserve(MAX_HEADER_LEN + bytes.len());
    encode_header(out, STRING_OFFSET, bytes.len());
    out.extend_from_slice(bytes);
}

pub fn encode_uint(out: &mut Vec<u8>, big_endian: &[u8]) {
    let start = big_endian.iter().position(|byte| *byte != 0).unwrap_or(big_endian.len());
    encode_bytes(out, &big_endian[start..]);
}

pub fn encode_list(out: &mut Vec<u8>, payload: &[u8]) {
    out.reserve(MAX_HEADER_LEN + payload.len());
    encode_header(out, LIST_OFFSET, payload.len());
    out.extend_from_slice(payload);
}

fn encode_header(out: &mut Vec<u8>, offset: u8, length: usize) {
    if length <= SHORT_PAYLOAD_LIMIT {
        out.push(offset + length as u8);
        return;
    }
    let length = length.to_be_bytes();
    let start = length.iter().position(|byte| *byte != 0).unwrap_or(length.len() - 1);
    out.push(offset + SHORT_PAYLOAD_LIMIT as u8 + (length.len() - start) as u8);
    out.extend_from_slice(&length[start..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    fn string(value: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_bytes(&mut out, value);
        out
    }

    fn uint(value: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_uint(&mut out, value);
        out
    }

    fn list(items: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_list(&mut out, &items.concat());
        out
    }

    #[test]
    fn test_encode_bytes() {
        assert_eq!(hex::encode(string(b"")), "80");
        assert_eq!(hex::encode(string(&[0x00])), "00");
        assert_eq!(hex::encode(string(&[0x01])), "01");
        assert_eq!(hex::encode(string(&[0x7f])), "7f");
        assert_eq!(hex::encode(string(&[0x80])), "8180");
        assert_eq!(hex::encode(string(&[0xff])), "81ff");
        assert_eq!(hex::encode(string(&[0x00, 0x00])), "820000");
        assert_eq!(hex::encode(string(b"dog")), "83646f67");
    }

    #[test]
    fn test_encode_bytes_length_boundaries() {
        let max_short = b"Lorem ipsum dolor sit amet, consectetur adipisicing eli";
        assert_eq!(hex::encode(string(max_short)), format!("b7{}", hex::encode(max_short)));
        let min_long = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit";
        assert_eq!(hex::encode(string(min_long)), format!("b838{}", hex::encode(min_long)));
        assert_eq!(string(&[0xaa; 255])[..2], [0xb8, 0xff]);
        assert_eq!(string(&[0xaa; 256])[..3], [0xb9, 0x01, 0x00]);
        assert_eq!(string(&[0xaa; 65_535])[..3], [0xb9, 0xff, 0xff]);
        assert_eq!(string(&[0xaa; 65_536])[..4], [0xba, 0x01, 0x00, 0x00]);
        assert_eq!(string(&[0xaa; 65_536]).len(), 65_540);
    }

    #[test]
    fn test_encode_uint() {
        assert_eq!(hex::encode(uint(&[])), "80");
        assert_eq!(hex::encode(uint(&0u64.to_be_bytes())), "80");
        assert_eq!(hex::encode(uint(&[0x00; 32])), "80");
        assert_eq!(hex::encode(uint(&1u64.to_be_bytes())), "01");
        assert_eq!(hex::encode(uint(&16u64.to_be_bytes())), "10");
        assert_eq!(hex::encode(uint(&79u64.to_be_bytes())), "4f");
        assert_eq!(hex::encode(uint(&127u64.to_be_bytes())), "7f");
        assert_eq!(hex::encode(uint(&128u64.to_be_bytes())), "8180");
        assert_eq!(hex::encode(uint(&1000u64.to_be_bytes())), "8203e8");
        assert_eq!(hex::encode(uint(&100_000u64.to_be_bytes())), "830186a0");
        assert_eq!(hex::encode(uint(&[0x00, 0x00, 0x01, 0x00])), "820100");
        assert_eq!(hex::encode(uint(&u64::MAX.to_be_bytes())), "88ffffffffffffffff");
        assert_eq!(hex::encode(uint(&u128::MAX.to_be_bytes())), format!("90{}", "ff".repeat(16)));
        assert_eq!(hex::encode(uint(&[0xff; 32])), format!("a0{}", "ff".repeat(32)));
        assert_eq!(hex::encode(uint(&hex::decode("102030405060708090a0b0c0d0e0f2").unwrap())), "8f102030405060708090a0b0c0d0e0f2");
        assert_eq!(
            hex::encode(uint(&hex::decode("0100020003000400050006000700080009000a000b000c000d000e01").unwrap())),
            "9c0100020003000400050006000700080009000a000b000c000d000e01"
        );
    }

    #[test]
    fn test_encode_list() {
        assert_eq!(hex::encode(list(&[])), "c0");
        assert_eq!(hex::encode(list(&[string(b"dog"), string(b"god"), string(b"cat")])), "cc83646f6783676f6483636174");
        assert_eq!(hex::encode(list(&[string(b"zw"), list(&[uint(&[4])]), uint(&[1])])), "c6827a77c10401");
        assert_eq!(hex::encode(list(&[list(&[list(&[]), list(&[])]), list(&[])])), "c4c2c0c0c0");
        assert_eq!(hex::encode(list(&[list(&[]), list(&[list(&[])]), list(&[list(&[]), list(&[list(&[])])])])), "c7c0c1c0c3c0c1c0");
    }

    #[test]
    fn test_encode_list_length_boundaries() {
        let words = ["asdf", "qwer", "zxcv", "asdf", "qwer", "zxcv", "asdf", "qwer", "zxcv", "asdf", "qwer"].map(|word| string(word.as_bytes()));
        let max_short = list(&words);
        assert_eq!(max_short[0], 0xf7);
        assert_eq!(max_short[1..], words.concat());

        let row = list(&[string(b"asdf"), string(b"qwer"), string(b"zxcv")]);
        assert_eq!(hex::encode(list(&[row.clone(), row.clone(), row.clone(), row.clone()])), format!("f840{}", hex::encode(row).repeat(4)));

        assert_eq!(list(&[string(&[0xaa; 55])])[..2], [0xf8, 0x38]);
        assert_eq!(list(&[string(&[0xaa; 1021])])[..3], [0xf9, 0x04, 0x00]);
    }
}
