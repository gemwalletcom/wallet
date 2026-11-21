use std::borrow::Cow;

const HEX: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Clone, Copy)]
pub(crate) struct EncodeSet(u128);

impl EncodeSet {
    const fn controls() -> Self {
        let mut bits = 1 << 0x7F;
        let mut byte = 0;
        while byte < 0x20 {
            bits |= 1 << byte;
            byte += 1;
        }
        Self(bits)
    }

    const fn add(self, byte: u8) -> Self {
        Self(self.0 | 1 << byte)
    }

    pub(crate) fn contains(self, byte: u8) -> bool {
        !byte.is_ascii() || self.0 & 1 << byte != 0
    }
}

pub(crate) const CONTROLS: EncodeSet = EncodeSet::controls();
pub(crate) const FRAGMENT: EncodeSet = CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
pub(crate) const PATH: EncodeSet = FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');
pub(crate) const USERINFO: EncodeSet = PATH.add(b'/').add(b':').add(b';').add(b'=').add(b'@').add(b'[').add(b'\\').add(b']').add(b'^').add(b'|');
pub(crate) const QUERY: EncodeSet = CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');
pub(crate) const SPECIAL_QUERY: EncodeSet = QUERY.add(b'\'');

pub(crate) fn encode_into(output: &mut String, input: &str, set: EncodeSet) {
    let mut rest = input;
    while let Some(index) = rest.bytes().position(|byte| set.contains(byte)) {
        let (unchanged, tail) = rest.split_at(index);
        output.push_str(unchanged);
        let (encoded, remaining) = tail.split_at(tail.chars().next().map_or(1, char::len_utf8));
        encoded.bytes().for_each(|byte| push_encoded_byte(output, byte));
        rest = remaining;
    }
    output.push_str(rest);
}

pub(crate) fn push_encoded_byte(output: &mut String, byte: u8) {
    output.push('%');
    output.push(char::from(HEX[usize::from(byte >> 4)]));
    output.push(char::from(HEX[usize::from(byte & 0x0F)]));
}

pub(crate) fn decode(input: &[u8]) -> Cow<'_, [u8]> {
    let Some(first) = (0..input.len()).find(|index| input[*index] == b'%' && escaped_byte(input, *index).is_some()) else {
        return Cow::Borrowed(input);
    };
    let mut output = Vec::with_capacity(input.len());
    output.extend_from_slice(&input[..first]);
    let mut index = first;
    while let Some(byte) = input.get(index) {
        match escaped_byte(input, index) {
            Some(decoded) => {
                output.push(decoded);
                index += 3;
            }
            None => {
                output.push(*byte);
                index += 1;
            }
        }
    }
    Cow::Owned(output)
}

fn escaped_byte(input: &[u8], index: usize) -> Option<u8> {
    match input.get(index..index + 3)? {
        [b'%', high, low] => Some(hex_value(*high)? << 4 | hex_value(*low)?),
        _ => None,
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
