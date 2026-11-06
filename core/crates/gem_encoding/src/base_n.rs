use crate::{EncodingError, EncodingType};

const INVALID: u8 = 0xff;

pub(crate) struct Alphabet<const GROUP_BYTES: usize, const GROUP_SYMBOLS: usize> {
    symbols: &'static [u8],
    values: [u8; 256],
}

impl<const GROUP_BYTES: usize, const GROUP_SYMBOLS: usize> Alphabet<GROUP_BYTES, GROUP_SYMBOLS> {
    const BITS: usize = GROUP_BYTES * 8 / GROUP_SYMBOLS;
    const MASK: u64 = (1 << Self::BITS) - 1;

    pub(crate) const fn new(symbols: &'static [u8]) -> Self {
        assert!(
            GROUP_BYTES <= 8 && GROUP_BYTES * 8 == GROUP_SYMBOLS * Self::BITS && symbols.len() == 1 << Self::BITS,
            "a group fits in 64 bits, holds whole symbols, and the alphabet has one symbol per value"
        );
        let mut values = [INVALID; 256];
        let mut index = 0;
        while index < symbols.len() {
            assert!(symbols[index].is_ascii(), "alphabet symbols are ASCII");
            values[symbols[index] as usize] = index as u8;
            index += 1;
        }
        Self { symbols, values }
    }

    pub(crate) fn encode(&self, bytes: &[u8], padded: bool) -> String {
        let mut encoded = Vec::with_capacity(bytes.len().div_ceil(GROUP_BYTES) * GROUP_SYMBOLS);
        let (groups, remainder) = bytes.as_chunks::<GROUP_BYTES>();
        for group in groups {
            self.encode_group(&mut encoded, group, GROUP_SYMBOLS);
        }
        if !remainder.is_empty() {
            let symbols = (remainder.len() * 8).div_ceil(Self::BITS);
            self.encode_group(&mut encoded, remainder, symbols);
            if padded {
                encoded.extend(std::iter::repeat_n(b'=', GROUP_SYMBOLS - symbols));
            }
        }
        String::from_utf8(encoded).expect("alphabet symbols are ASCII")
    }

    pub(crate) fn decode(&self, value: &[u8], encoding: EncodingType) -> Result<Vec<u8>, EncodingError> {
        let invalid = |message: &str| EncodingError::Invalid(encoding, message.to_string());
        let remainder_width = value.len() % GROUP_SYMBOLS * Self::BITS;
        if remainder_width % 8 >= Self::BITS {
            return Err(invalid("invalid length"));
        }
        let mut decoded = Vec::with_capacity(value.len() / GROUP_SYMBOLS * GROUP_BYTES + remainder_width / 8);
        let (groups, remainder) = value.as_chunks::<GROUP_SYMBOLS>();
        for group in groups {
            let packed = self.pack(group).ok_or_else(|| invalid("invalid symbol"))?;
            decoded.extend((0..GROUP_BYTES).rev().map(|index| (packed >> (8 * index)) as u8));
        }
        if !remainder.is_empty() {
            let packed = self.pack(remainder).ok_or_else(|| invalid("invalid symbol"))?;
            let trailing = remainder_width % 8;
            if packed & ((1 << trailing) - 1) != 0 {
                return Err(invalid("non-zero trailing bits"));
            }
            decoded.extend((0..remainder_width / 8).rev().map(|index| (packed >> (trailing + 8 * index)) as u8));
        }
        Ok(decoded)
    }

    fn encode_group(&self, encoded: &mut Vec<u8>, group: &[u8], symbols: usize) {
        let packed = group.iter().fold(0u64, |packed, byte| packed << 8 | u64::from(*byte)) << (symbols * Self::BITS - group.len() * 8);
        encoded.extend((0..symbols).rev().map(|index| self.symbols[((packed >> (Self::BITS * index)) & Self::MASK) as usize]));
    }

    fn pack(&self, group: &[u8]) -> Option<u64> {
        group.iter().try_fold(0u64, |packed, symbol| match self.values[usize::from(*symbol)] {
            INVALID => None,
            value => Some(packed << Self::BITS | u64::from(value)),
        })
    }
}
