#![no_main]

use gem_encoding::borsh::{BorshDeserialize, BorshSerialize, DecodeError, Reader};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, PartialEq)]
enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I32(i32),
    I64(i64),
    I128(i128),
    Bool(bool),
    Bytes([u8; 32]),
    Words([u64; 2]),
    String(String),
    Option(Option<u64>),
    Vec(Vec<u16>),
    Pair((u8, i64)),
}

impl BorshDeserialize for Value {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(match reader.read::<u8>()? {
            0 => Self::U8(reader.read()?),
            1 => Self::U16(reader.read()?),
            2 => Self::U32(reader.read()?),
            3 => Self::U64(reader.read()?),
            4 => Self::U128(reader.read()?),
            5 => Self::I32(reader.read()?),
            6 => Self::I64(reader.read()?),
            7 => Self::I128(reader.read()?),
            8 => Self::Bool(reader.read()?),
            9 => Self::Bytes(reader.read()?),
            10 => Self::Words(reader.read()?),
            11 => Self::String(reader.read()?),
            12 => Self::Option(reader.read()?),
            13 => Self::Vec(reader.read()?),
            14 => Self::Pair(reader.read()?),
            tag => return Err(DecodeError::UnexpectedTag(tag)),
        })
    }
}

impl BorshSerialize for Value {
    fn serialize(&self, writer: &mut Vec<u8>) {
        match self {
            Self::U8(value) => (0u8, *value).serialize(writer),
            Self::U16(value) => (1u8, *value).serialize(writer),
            Self::U32(value) => (2u8, *value).serialize(writer),
            Self::U64(value) => (3u8, *value).serialize(writer),
            Self::U128(value) => (4u8, *value).serialize(writer),
            Self::I32(value) => (5u8, *value).serialize(writer),
            Self::I64(value) => (6u8, *value).serialize(writer),
            Self::I128(value) => (7u8, *value).serialize(writer),
            Self::Bool(value) => (8u8, *value).serialize(writer),
            Self::Bytes(value) => writer.extend([[9].as_slice(), value].concat()),
            Self::Words(values) => {
                writer.push(10);
                values.iter().for_each(|value| value.serialize(writer));
            }
            Self::String(value) => {
                writer.push(11);
                (value.len() as u32).serialize(writer);
                writer.extend_from_slice(value.as_bytes());
            }
            Self::Option(value) => (12u8, *value).serialize(writer),
            Self::Vec(values) => (13u8, values.clone()).serialize(writer),
            Self::Pair(value) => (14u8, *value).serialize(writer),
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let mut rest = data;
    match <Vec<Value>>::deserialize(&mut rest) {
        Ok(values) => {
            assert_eq!(values.to_vec(), data[..data.len() - rest.len()], "decoded values encode to different bytes");
            let whole = if rest.is_empty() { Ok(values) } else { Err(DecodeError::TrailingBytes) };
            assert_eq!(<Vec<Value>>::try_from_slice(data), whole);
        }
        Err(error) => assert_eq!(<Vec<Value>>::try_from_slice(data), Err(error)),
    }
});
