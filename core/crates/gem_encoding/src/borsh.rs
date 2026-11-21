use std::{error::Error, fmt, str};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedLength,
    InvalidBool(u8),
    InvalidOption(u8),
    UnexpectedTag(u8),
    InvalidUtf8,
    TrailingBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedLength => write!(f, "Unexpected length of input"),
            Self::InvalidBool(value) => write!(f, "Invalid bool representation: {value}"),
            Self::InvalidOption(value) => write!(f, "Invalid Option representation: {value}. The first byte must be 0 or 1"),
            Self::UnexpectedTag(tag) => write!(f, "Unexpected variant tag: {tag}"),
            Self::InvalidUtf8 => write!(f, "invalid utf-8 sequence"),
            Self::TrailingBytes => write!(f, "Not all bytes read"),
        }
    }
}

impl Error for DecodeError {}

pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn decode<T>(data: &'a [u8], read: impl FnOnce(&mut Self) -> Result<T, DecodeError>) -> Result<T, DecodeError> {
        let mut reader = Self { data };
        let value = read(&mut reader)?;
        if reader.data.is_empty() { Ok(value) } else { Err(DecodeError::TrailingBytes) }
    }

    pub fn decode_prefix<T>(data: &'a [u8], read: impl FnOnce(&mut Self) -> Result<T, DecodeError>) -> Result<T, DecodeError> {
        read(&mut Self { data })
    }

    #[inline]
    pub fn bytes<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let (bytes, rest) = self.data.split_first_chunk().ok_or(DecodeError::UnexpectedLength)?;
        self.data = rest;
        Ok(*bytes)
    }

    #[inline]
    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        let (byte, rest) = self.data.split_first().ok_or(DecodeError::UnexpectedLength)?;
        self.data = rest;
        Ok(*byte)
    }

    #[inline]
    pub fn u16(&mut self) -> Result<u16, DecodeError> {
        self.bytes().map(u16::from_le_bytes)
    }

    #[inline]
    pub fn u32(&mut self) -> Result<u32, DecodeError> {
        self.bytes().map(u32::from_le_bytes)
    }

    #[inline]
    pub fn u64(&mut self) -> Result<u64, DecodeError> {
        self.bytes().map(u64::from_le_bytes)
    }

    #[inline]
    pub fn u128(&mut self) -> Result<u128, DecodeError> {
        self.bytes().map(u128::from_le_bytes)
    }

    #[inline]
    pub fn i32(&mut self) -> Result<i32, DecodeError> {
        self.bytes().map(i32::from_le_bytes)
    }

    #[inline]
    pub fn i64(&mut self) -> Result<i64, DecodeError> {
        self.bytes().map(i64::from_le_bytes)
    }

    #[inline]
    pub fn i128(&mut self) -> Result<i128, DecodeError> {
        self.bytes().map(i128::from_le_bytes)
    }

    #[inline]
    pub fn bool(&mut self) -> Result<bool, DecodeError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(DecodeError::InvalidBool(value)),
        }
    }

    #[inline]
    pub fn string(&mut self) -> Result<String, DecodeError> {
        let len = self.u32()? as usize;
        let (bytes, rest) = self.data.split_at_checked(len).ok_or(DecodeError::UnexpectedLength)?;
        self.data = rest;
        str::from_utf8(bytes).map(str::to_owned).map_err(|_| DecodeError::InvalidUtf8)
    }

    pub fn option<T>(&mut self, read: impl FnOnce(&mut Self) -> Result<T, DecodeError>) -> Result<Option<T>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => read(self).map(Some),
            value => Err(DecodeError::InvalidOption(value)),
        }
    }

    pub fn array<T: Copy + Default, const N: usize>(&mut self, mut read: impl FnMut(&mut Self) -> Result<T, DecodeError>) -> Result<[T; N], DecodeError> {
        let mut values = [T::default(); N];
        for value in &mut values {
            *value = read(self)?;
        }
        Ok(values)
    }

    pub fn list<T>(&mut self, len: usize, mut read: impl FnMut(&mut Self) -> Result<T, DecodeError>) -> Result<Vec<T>, DecodeError> {
        let mut values = Vec::with_capacity(len.min(self.data.len() / size_of::<T>().max(1)));
        for _ in 0..len {
            values.push(read(self)?);
        }
        Ok(values)
    }

    pub fn vec<T>(&mut self, read: impl FnMut(&mut Self) -> Result<T, DecodeError>) -> Result<Vec<T>, DecodeError> {
        let len = self.u32()? as usize;
        self.list(len, read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        assert_eq!(Reader::decode(&[42], Reader::u8), Ok(42));
        assert_eq!(Reader::decode(&[0x34, 0x12], Reader::u16), Ok(0x1234));
        assert_eq!(Reader::decode(&[0x78, 0x56, 0x34, 0x12], Reader::u32), Ok(0x1234_5678));
        assert_eq!(Reader::decode(&100u64.to_le_bytes(), Reader::u64), Ok(100));
        assert_eq!(Reader::decode(&[0xff; 8], Reader::u64), Ok(u64::MAX));
        assert_eq!(Reader::decode(&[0xff; 16], Reader::u128), Ok(u128::MAX));
        assert_eq!(Reader::decode(&[0x9c, 0xff, 0xff, 0xff], Reader::i32), Ok(-100));
        assert_eq!(Reader::decode(&[0, 0, 0, 0, 0, 0, 0, 0x80], Reader::i64), Ok(i64::MIN));
        assert_eq!(Reader::decode(&i128::MIN.to_le_bytes(), Reader::i128), Ok(i128::MIN));
        assert_eq!(Reader::decode(&[1, 2, 3], Reader::bytes::<3>), Ok([1, 2, 3]));
        assert_eq!(Reader::decode(&[], Reader::bytes::<0>), Ok([]));
        assert_eq!(Reader::decode(&[], Reader::u8), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[0; 7], Reader::u64), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[1, 0], |reader| Ok((reader.u64()?, reader.u32()?))), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[1, 2], Reader::bytes::<3>), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[1, 0, 0, 0, 32, 32], |reader| reader.vec(Reader::u8)), Err(DecodeError::TrailingBytes));
    }

    #[test]
    fn test_decode_prefix() {
        assert_eq!(Reader::decode_prefix(&[1, 0, 0, 0, 32, 32], |reader| reader.vec(Reader::u8)), Ok(vec![32]));
        assert_eq!(Reader::decode_prefix(&[1, 0], Reader::u16), Ok(1));
        assert_eq!(Reader::decode_prefix(&[1], Reader::u16), Err(DecodeError::UnexpectedLength));
    }

    #[test]
    fn test_bool() {
        assert_eq!(Reader::decode(&[0], Reader::bool), Ok(false));
        assert_eq!(Reader::decode(&[1], Reader::bool), Ok(true));
        for value in 2u8..=255 {
            assert_eq!(Reader::decode(&[value], Reader::bool), Err(DecodeError::InvalidBool(value)));
        }
    }

    #[test]
    fn test_option() {
        assert_eq!(Reader::decode(&[0], |reader| reader.option(Reader::u8)), Ok(None));
        assert_eq!(Reader::decode(&[1, 32], |reader| reader.option(Reader::u8)), Ok(Some(32)));
        assert_eq!(Reader::decode(&[1], |reader| reader.option(Reader::u8)), Err(DecodeError::UnexpectedLength));
        for value in 2u8..=255 {
            assert_eq!(Reader::decode(&[value, 32], |reader| reader.option(Reader::u8)), Err(DecodeError::InvalidOption(value)));
        }
    }

    #[test]
    fn test_string() {
        for value in ["", "a", "hello world", "💩", &"x".repeat(1024), &"hello world!".repeat(1000)] {
            let bytes = [(value.len() as u32).to_le_bytes().as_slice(), value.as_bytes()].concat();
            assert_eq!(Reader::decode(&bytes, Reader::string), Ok(value.to_string()));
        }
        assert_eq!(Reader::decode(&[255; 4], Reader::string), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[255, 255, 255, 255, 32, 32], Reader::string), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[1, 0, 0, 0, 0xc0], Reader::string), Err(DecodeError::InvalidUtf8));
    }

    #[test]
    fn test_vec() {
        assert_eq!(Reader::decode(&[0; 4], |reader| reader.vec(Reader::u64)), Ok(vec![]));
        assert_eq!(Reader::decode(&[2, 0, 0, 0, 1, 0, 2, 0], |reader| reader.vec(Reader::u16)), Ok(vec![1, 2]));
        assert_eq!(Reader::decode(&[255; 4], |reader| reader.vec(Reader::u64)), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[255, 255, 255, 255, 32, 32], |reader| reader.vec(Reader::bytes::<32>)), Err(DecodeError::UnexpectedLength));
        assert_eq!(Reader::decode(&[1, 2, 3, 4], |reader| reader.list(2, Reader::u16)), Ok(vec![0x0201, 0x0403]));
        assert_eq!(Reader::decode(&[1, 2, 3, 4], |reader| reader.array(Reader::u16)), Ok([0x0201, 0x0403]));
        assert_eq!(Reader::decode(&[1, 2, 3], |reader| reader.array::<_, 2>(Reader::u16)), Err(DecodeError::UnexpectedLength));
    }

    #[test]
    fn test_display() {
        assert_eq!(DecodeError::UnexpectedLength.to_string(), "Unexpected length of input");
        assert_eq!(DecodeError::InvalidBool(2).to_string(), "Invalid bool representation: 2");
        assert_eq!(DecodeError::InvalidOption(255).to_string(), "Invalid Option representation: 255. The first byte must be 0 or 1");
        assert_eq!(DecodeError::UnexpectedTag(123).to_string(), "Unexpected variant tag: 123");
        assert_eq!(DecodeError::InvalidUtf8.to_string(), "invalid utf-8 sequence");
        assert_eq!(DecodeError::TrailingBytes.to_string(), "Not all bytes read");
    }
}
