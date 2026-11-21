use std::{array, error::Error, fmt, str};

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

impl Reader<'_> {
    #[inline]
    pub fn read<T: BorshDeserialize>(&mut self) -> Result<T, DecodeError> {
        T::deserialize_reader(self)
    }

    #[inline]
    fn list<T: BorshDeserialize>(&mut self, len: usize) -> Result<Vec<T>, DecodeError> {
        let mut values = Vec::with_capacity(len.min(self.data.len() / size_of::<T>().max(1)));
        for _ in 0..len {
            values.push(self.read()?);
        }
        Ok(values)
    }

    #[inline]
    pub fn skip(&mut self, len: usize) -> Result<&mut Self, DecodeError> {
        self.data = self.data.get(len..).ok_or(DecodeError::UnexpectedLength)?;
        Ok(self)
    }

    #[inline]
    pub fn bytes<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let (bytes, rest) = self.data.split_first_chunk().ok_or(DecodeError::UnexpectedLength)?;
        self.data = rest;
        Ok(*bytes)
    }
}

pub trait BorshDeserialize: Sized {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError>;

    fn deserialize(data: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader { data };
        let value = Self::deserialize_reader(&mut reader)?;
        *data = reader.data;
        Ok(value)
    }

    fn try_from_slice(data: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader { data };
        let value = Self::deserialize_reader(&mut reader)?;
        if reader.data.is_empty() { Ok(value) } else { Err(DecodeError::TrailingBytes) }
    }
}

pub trait BorshSerialize {
    fn serialize(&self, writer: &mut Vec<u8>);

    fn to_vec(&self) -> Vec<u8> {
        let mut writer = Vec::new();
        self.serialize(&mut writer);
        writer
    }
}

macro_rules! integers {
    ($($integer:ty),*) => {
        $(
            impl BorshDeserialize for $integer {
                #[inline]
                fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
                    reader.bytes().map(<$integer>::from_le_bytes)
                }
            }

            impl BorshSerialize for $integer {
                #[inline]
                fn serialize(&self, writer: &mut Vec<u8>) {
                    writer.extend_from_slice(&self.to_le_bytes());
                }
            }
        )*
    };
}

integers!(u8, u16, u32, u64, u128, i32, i64, i128);

impl BorshDeserialize for bool {
    #[inline]
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(DecodeError::InvalidBool(value)),
        }
    }
}

impl BorshSerialize for bool {
    #[inline]
    fn serialize(&self, writer: &mut Vec<u8>) {
        writer.push(u8::from(*self));
    }
}

impl<T: BorshDeserialize + Default, const N: usize> BorshDeserialize for [T; N] {
    #[inline]
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let mut values: [T; N] = array::from_fn(|_| T::default());
        for value in &mut values {
            *value = reader.read()?;
        }
        Ok(values)
    }
}

impl<A: BorshDeserialize, B: BorshDeserialize> BorshDeserialize for (A, B) {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok((reader.read()?, reader.read()?))
    }
}

impl<A: BorshSerialize, B: BorshSerialize> BorshSerialize for (A, B) {
    fn serialize(&self, writer: &mut Vec<u8>) {
        self.0.serialize(writer);
        self.1.serialize(writer);
    }
}

impl<T: BorshDeserialize> BorshDeserialize for Option<T> {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(None),
            1 => reader.read().map(Some),
            value => Err(DecodeError::InvalidOption(value)),
        }
    }
}

impl<T: BorshSerialize> BorshSerialize for Option<T> {
    fn serialize(&self, writer: &mut Vec<u8>) {
        match self {
            None => writer.push(0),
            Some(value) => {
                writer.push(1);
                value.serialize(writer);
            }
        }
    }
}

impl<T: BorshDeserialize> BorshDeserialize for Vec<T> {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let len = reader.read::<u32>()?;
        reader.list(len as usize)
    }
}

impl<T: BorshSerialize> BorshSerialize for Vec<T> {
    fn serialize(&self, writer: &mut Vec<u8>) {
        (self.len() as u32).serialize(writer);
        self.iter().for_each(|value| value.serialize(writer));
    }
}

impl BorshDeserialize for String {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let len = reader.read::<u32>()? as usize;
        let (bytes, rest) = reader.data.split_at_checked(len).ok_or(DecodeError::UnexpectedLength)?;
        reader.data = rest;
        str::from_utf8(bytes).map(str::to_owned).map_err(|_| DecodeError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_slice() {
        assert_eq!(u8::try_from_slice(&[42]), Ok(42));
        assert_eq!(u16::try_from_slice(&[0x34, 0x12]), Ok(0x1234));
        assert_eq!(u32::try_from_slice(&[0x78, 0x56, 0x34, 0x12]), Ok(0x1234_5678));
        assert_eq!(u64::try_from_slice(&100u64.to_le_bytes()), Ok(100));
        assert_eq!(u64::try_from_slice(&[0xff; 8]), Ok(u64::MAX));
        assert_eq!(u128::try_from_slice(&[0xff; 16]), Ok(u128::MAX));
        assert_eq!(i32::try_from_slice(&[0x9c, 0xff, 0xff, 0xff]), Ok(-100));
        assert_eq!(i64::try_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0x80]), Ok(i64::MIN));
        assert_eq!(i128::try_from_slice(&i128::MIN.to_le_bytes()), Ok(i128::MIN));
        assert_eq!(<[u8; 3]>::try_from_slice(&[1, 2, 3]), Ok([1, 2, 3]));
        assert_eq!(<[u8; 0]>::try_from_slice(&[]), Ok([]));
        assert_eq!(<[u64; 2]>::try_from_slice(&[[1u8; 8], [2u8; 8]].concat()), Ok([0x0101_0101_0101_0101, 0x0202_0202_0202_0202]));
        assert_eq!(<(u8, u16)>::try_from_slice(&[1, 2, 0]), Ok((1, 2)));
        assert_eq!(u8::try_from_slice(&[]), Err(DecodeError::UnexpectedLength));
        assert_eq!(u64::try_from_slice(&[0; 7]), Err(DecodeError::UnexpectedLength));
        assert_eq!(<(u64, u32)>::try_from_slice(&[1, 0]), Err(DecodeError::UnexpectedLength));
        assert_eq!(<[u8; 3]>::try_from_slice(&[1, 2]), Err(DecodeError::UnexpectedLength));
        assert_eq!(<[u64; 2]>::try_from_slice(&[0; 15]), Err(DecodeError::UnexpectedLength));
        assert_eq!(<Vec<u8>>::try_from_slice(&[1, 0, 0, 0, 32, 32]), Err(DecodeError::TrailingBytes));
    }

    #[derive(Debug, PartialEq)]
    struct Skipped(u16);

    impl BorshDeserialize for Skipped {
        fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
            reader.skip(2)?.read().map(Self)
        }
    }

    #[test]
    fn test_skip() {
        assert_eq!(Skipped::try_from_slice(&[9, 9, 1, 0]), Ok(Skipped(1)));
        assert_eq!(Skipped::try_from_slice(&[9, 9, 1]), Err(DecodeError::UnexpectedLength));
        assert_eq!(Skipped::try_from_slice(&[9]), Err(DecodeError::UnexpectedLength));
        assert_eq!(Skipped::try_from_slice(&[9, 9, 1, 0, 7]), Err(DecodeError::TrailingBytes));
    }

    #[test]
    fn test_deserialize() {
        let mut data = [1, 0, 0, 0, 32, 32].as_slice();

        assert_eq!(<Vec<u8>>::deserialize(&mut data), Ok(vec![32]));
        assert_eq!(data, [32]);
        assert_eq!(u16::deserialize(&mut [1, 0, 7].as_slice()), Ok(1));
        assert_eq!(u16::deserialize(&mut [1].as_slice()), Err(DecodeError::UnexpectedLength));
    }

    #[test]
    fn test_bool() {
        assert_eq!(bool::try_from_slice(&[0]), Ok(false));
        assert_eq!(bool::try_from_slice(&[1]), Ok(true));
        for value in 2u8..=255 {
            assert_eq!(bool::try_from_slice(&[value]), Err(DecodeError::InvalidBool(value)));
        }
    }

    #[test]
    fn test_option() {
        assert_eq!(<Option<u8>>::try_from_slice(&[0]), Ok(None));
        assert_eq!(<Option<u8>>::try_from_slice(&[1, 32]), Ok(Some(32)));
        assert_eq!(<Option<u8>>::try_from_slice(&[1]), Err(DecodeError::UnexpectedLength));
        for value in 2u8..=255 {
            assert_eq!(<Option<u8>>::try_from_slice(&[value, 32]), Err(DecodeError::InvalidOption(value)));
        }
    }

    #[test]
    fn test_string() {
        for value in ["", "a", "hello world", "💩", &"x".repeat(1024), &"hello world!".repeat(1000)] {
            let bytes = [(value.len() as u32).to_le_bytes().as_slice(), value.as_bytes()].concat();
            assert_eq!(String::try_from_slice(&bytes), Ok(value.to_string()));
        }
        assert_eq!(String::try_from_slice(&[255; 4]), Err(DecodeError::UnexpectedLength));
        assert_eq!(String::try_from_slice(&[255, 255, 255, 255, 32, 32]), Err(DecodeError::UnexpectedLength));
        assert_eq!(String::try_from_slice(&[1, 0, 0, 0, 0xc0]), Err(DecodeError::InvalidUtf8));
    }

    #[test]
    fn test_vec() {
        assert_eq!(<Vec<u64>>::try_from_slice(&[0; 4]), Ok(vec![]));
        assert_eq!(<Vec<u16>>::try_from_slice(&[2, 0, 0, 0, 1, 0, 2, 0]), Ok(vec![1, 2]));
        assert_eq!(<Vec<u64>>::try_from_slice(&[255; 4]), Err(DecodeError::UnexpectedLength));
        assert_eq!(<Vec<[u8; 32]>>::try_from_slice(&[255, 255, 255, 255, 32, 32]), Err(DecodeError::UnexpectedLength));
    }

    #[test]
    fn test_to_vec() {
        assert_eq!(42u8.to_vec(), [42]);
        assert_eq!(0x1234u16.to_vec(), [0x34, 0x12]);
        assert_eq!((-100i32).to_vec(), [0x9c, 0xff, 0xff, 0xff]);
        assert_eq!(u128::MAX.to_vec(), [0xff; 16]);
        assert_eq!(true.to_vec(), [1]);
        assert_eq!(false.to_vec(), [0]);
        assert_eq!(None::<u64>.to_vec(), [0]);
        assert_eq!(Some(7u8).to_vec(), [1, 7]);
        assert_eq!(vec![(1u8, 2u8), (3, 4)].to_vec(), [2, 0, 0, 0, 1, 2, 3, 4]);
        assert_eq!(Vec::<u8>::new().to_vec(), [0, 0, 0, 0]);
    }
}
