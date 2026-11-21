#![no_main]

use gem_encoding::{
    borsh::{DecodeError, Reader},
    encode_base64,
};
use gem_solana::{
    metaplex::decode_metadata,
    metaplex_core::decode_asset,
    models::{Clock, Mint, TokenAccount},
};
use libfuzzer_sys::fuzz_target;

const LIST_LEN: usize = 3;

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
    String(String),
    Option(Option<u64>),
    Vec(Vec<u16>),
    List(Vec<i64>),
    Array([u32; 2]),
}

fn read(reader: &mut Reader<'_>, kind: u8) -> Result<Value, DecodeError> {
    Ok(match kind % 15 {
        0 => Value::U8(reader.u8()?),
        1 => Value::U16(reader.u16()?),
        2 => Value::U32(reader.u32()?),
        3 => Value::U64(reader.u64()?),
        4 => Value::U128(reader.u128()?),
        5 => Value::I32(reader.i32()?),
        6 => Value::I64(reader.i64()?),
        7 => Value::I128(reader.i128()?),
        8 => Value::Bool(reader.bool()?),
        9 => Value::Bytes(reader.bytes()?),
        10 => Value::String(reader.string()?),
        11 => Value::Option(reader.option(Reader::u64)?),
        12 => Value::Vec(reader.vec(Reader::u16)?),
        13 => Value::List(reader.list(LIST_LEN, Reader::i64)?),
        _ => Value::Array(reader.array(Reader::u32)?),
    })
}

fn encode(value: &Value) -> Vec<u8> {
    match value {
        Value::U8(value) => vec![*value],
        Value::U16(value) => value.to_le_bytes().to_vec(),
        Value::U32(value) => value.to_le_bytes().to_vec(),
        Value::U64(value) => value.to_le_bytes().to_vec(),
        Value::U128(value) => value.to_le_bytes().to_vec(),
        Value::I32(value) => value.to_le_bytes().to_vec(),
        Value::I64(value) => value.to_le_bytes().to_vec(),
        Value::I128(value) => value.to_le_bytes().to_vec(),
        Value::Bool(value) => vec![u8::from(*value)],
        Value::Bytes(value) => value.to_vec(),
        Value::String(value) => [(value.len() as u32).to_le_bytes().as_slice(), value.as_bytes()].concat(),
        Value::Option(None) => vec![0],
        Value::Option(Some(value)) => [[1].as_slice(), &value.to_le_bytes()].concat(),
        Value::Vec(values) => [(values.len() as u32).to_le_bytes().to_vec(), values.iter().flat_map(|value| value.to_le_bytes()).collect()].concat(),
        Value::List(values) => values.iter().flat_map(|value| value.to_le_bytes()).collect(),
        Value::Array(values) => values.iter().flat_map(|value| value.to_le_bytes()).collect(),
    }
}

fuzz_target!(|input: (&[u8], &[u8])| {
    let (schema, data) = input;
    let read_all = |reader: &mut Reader<'_>| schema.iter().map(|kind| read(reader, *kind)).collect::<Result<Vec<_>, _>>();
    match Reader::decode_prefix(data, read_all) {
        Ok(values) => {
            let encoded: Vec<u8> = values.iter().flat_map(encode).collect();
            assert_eq!(data.get(..encoded.len()), Some(encoded.as_slice()), "decoded values encode to different bytes");
            let whole = if encoded.len() == data.len() { Ok(values) } else { Err(DecodeError::TrailingBytes) };
            assert_eq!(Reader::decode(data, read_all), whole);
        }
        Err(error) => assert_eq!(Reader::decode(data, read_all), Err(error)),
    }

    let _ = Clock::decode(data);
    let _ = Mint::decode(data);
    let _ = TokenAccount::decode(data);
    let encoded = encode_base64(data);
    let _ = decode_metadata(&encoded);
    let _ = decode_asset(&encoded);
});
