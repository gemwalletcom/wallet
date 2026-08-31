const _: () = assert!(size_of::<usize>() <= size_of::<u64>());

pub fn encode_varint(value: usize) -> Vec<u8> {
    let value = value as u64;
    if value < 0xfd {
        vec![value as u8]
    } else if value <= 0xffff {
        let b = (value as u16).to_le_bytes();
        vec![0xfd, b[0], b[1]]
    } else if value <= 0xffffffff {
        let b = (value as u32).to_le_bytes();
        vec![0xfe, b[0], b[1], b[2], b[3]]
    } else {
        let b = value.to_le_bytes();
        vec![0xff, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
    }
}

pub(crate) fn varint_len(value: usize) -> usize {
    match value {
        0..=0xfc => 1,
        0xfd..=0xffff => 3,
        0x1_0000..=0xffff_ffff => 5,
        _ => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint() {
        assert_eq!(encode_varint(0), vec![0]);
        assert_eq!(encode_varint(252), vec![252]);
        assert_eq!(encode_varint(253), vec![0xfd, 253, 0]);
        assert_eq!(encode_varint(0xffff), vec![0xfd, 0xff, 0xff]);
        assert_eq!(encode_varint(0x10000), vec![0xfe, 0, 0, 1, 0]);
        assert_eq!(encode_varint(0x1_0000_0000), vec![0xff, 0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(encode_varint(usize::MAX), vec![0xff; 9]);
        assert_eq!(varint_len(0), 1);
        assert_eq!(varint_len(0xfc), 1);
        assert_eq!(varint_len(0xfd), 3);
        assert_eq!(varint_len(0xffff), 3);
        assert_eq!(varint_len(0x1_0000), 5);
        assert_eq!(varint_len(0xffff_ffff), 5);
        assert_eq!(varint_len(0x1_0000_0000), 9);
    }
}
