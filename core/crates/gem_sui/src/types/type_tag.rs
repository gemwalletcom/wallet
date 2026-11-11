use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::Address;
use crate::SuiError;

const MAX_IDENTIFIER_LENGTH: usize = 128;
pub(super) const MAX_TYPE_TAG_DEPTH: usize = 128;
pub(super) const MAX_TYPE_TAG_NODES: usize = 256;
const PATH_SEPARATOR: &str = "::";
const VECTOR_PREFIX: &str = "vector<";
const PRIMITIVE_TYPES: [TypeTag; 9] = [TypeTag::U8, TypeTag::U16, TypeTag::U32, TypeTag::U64, TypeTag::U128, TypeTag::U256, TypeTag::Bool, TypeTag::Address, TypeTag::Signer];

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeTag>),
    Struct(Box<StructTag>),
    U16,
    U32,
    U256,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct StructTag {
    pub address: Address,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTag>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Identifier(String);

impl Identifier {
    pub fn new(value: &str) -> Result<Self, SuiError> {
        match identifier_length(value.as_bytes()) {
            Some(length) if length == value.len() => Ok(Self(value.to_string())),
            Some(_) | None => Err(SuiError::invalid_input(format!("Invalid Sui identifier {value}"))),
        }
    }
}

impl TryFrom<String> for Identifier {
    type Error = SuiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Identifier> for String {
    fn from(identifier: Identifier) -> Self {
        identifier.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<StructTag> for TypeTag {
    fn from(value: StructTag) -> Self {
        Self::Struct(Box::new(value))
    }
}

impl fmt::Display for StructTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{PATH_SEPARATOR}{}{PATH_SEPARATOR}{}", self.address, self.module, self.name)?;
        if self.type_params.is_empty() {
            return Ok(());
        }
        let type_params = self.type_params.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
        write!(formatter, "<{type_params}>")
    }
}

impl fmt::Display for TypeTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vector(element) => write!(formatter, "{VECTOR_PREFIX}{element}>"),
            Self::Struct(struct_tag) => struct_tag.fmt(formatter),
            Self::Bool => formatter.write_str("bool"),
            Self::U8 => formatter.write_str("u8"),
            Self::U16 => formatter.write_str("u16"),
            Self::U32 => formatter.write_str("u32"),
            Self::U64 => formatter.write_str("u64"),
            Self::U128 => formatter.write_str("u128"),
            Self::U256 => formatter.write_str("u256"),
            Self::Address => formatter.write_str("address"),
            Self::Signer => formatter.write_str("signer"),
        }
    }
}

impl FromStr for TypeTag {
    type Err = SuiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parser = TypeTagParser { rest: value, nodes: 0 };
        match parser.type_tag(0) {
            Some(type_tag) if parser.rest.is_empty() => Ok(type_tag),
            Some(_) | None => Err(SuiError::invalid_input(format!("Invalid Sui type {value}"))),
        }
    }
}

struct TypeTagParser<'a> {
    rest: &'a str,
    nodes: usize,
}

impl TypeTagParser<'_> {
    fn type_tag(&mut self, depth: usize) -> Option<TypeTag> {
        self.nodes += 1;
        if self.nodes > MAX_TYPE_TAG_NODES || depth >= MAX_TYPE_TAG_DEPTH {
            return None;
        }
        if let Some(primitive) = PRIMITIVE_TYPES.iter().find(|primitive| self.take(&primitive.to_string())) {
            return Some(primitive.clone());
        }
        if self.take(VECTOR_PREFIX) {
            let element = self.type_tag(depth + 1)?;
            return self.take(">").then(|| TypeTag::Vector(Box::new(element)));
        }
        self.struct_tag(depth).map(TypeTag::from)
    }

    fn struct_tag(&mut self, depth: usize) -> Option<StructTag> {
        let address = self.address()?;
        let module = self.path_identifier()?;
        let name = self.path_identifier()?;
        let type_params = if self.take("<") { self.type_params(depth + 1)? } else { vec![] };
        Some(StructTag { address, module, name, type_params })
    }

    fn type_params(&mut self, depth: usize) -> Option<Vec<TypeTag>> {
        let mut type_params = vec![];
        loop {
            self.skip_spaces();
            type_params.push(self.type_tag(depth)?);
            self.skip_spaces();
            if self.take(">") {
                return Some(type_params);
            }
            if !self.take(",") {
                return None;
            }
        }
    }

    fn address(&mut self) -> Option<Address> {
        let digits = self.rest.strip_prefix("0x")?;
        let length = digits.bytes().take(Address::LENGTH * 2).take_while(u8::is_ascii_hexdigit).count();
        let address = digits[..length].parse().ok()?;
        self.rest = &digits[length..];
        Some(address)
    }

    fn path_identifier(&mut self) -> Option<Identifier> {
        if !self.take(PATH_SEPARATOR) {
            return None;
        }
        let length = identifier_length(self.rest.as_bytes())?;
        let (identifier, rest) = self.rest.split_at(length);
        self.rest = rest;
        Some(Identifier(identifier.to_string()))
    }

    fn take(&mut self, token: &str) -> bool {
        match self.rest.strip_prefix(token) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    fn skip_spaces(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t']);
    }
}

fn identifier_length(bytes: &[u8]) -> Option<usize> {
    let mut length = 0;
    while length < bytes.len() && length < MAX_IDENTIFIER_LENGTH && (bytes[length].is_ascii_alphanumeric() || bytes[length] == b'_') {
        length += 1;
    }
    if length == 0 {
        return None;
    }
    match bytes[0] {
        b'_' if length > 1 => Some(length),
        first if first.is_ascii_alphabetic() => Some(length),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_tag_from_str() {
        let displayed = |value: &str| value.parse::<TypeTag>().unwrap().to_string();
        let address = |digit: &str| format!("0x{digit:0>64}");

        for primitive in ["u8", "u16", "u32", "u64", "u128", "u256", "bool", "address", "signer", "vector<address>", "vector<vector<u256>>"] {
            assert_eq!(displayed(primitive), primitive);
        }
        assert_eq!(displayed("0x2::sui::SUI"), format!("{}::sui::SUI", address("2")));
        assert_eq!(displayed("0x00a::_m::_T1"), format!("{}::_m::_T1", address("a")));
        assert_eq!(
            displayed("0x1::pair::Pair<0x2::sui::SUI,   0xabc::token::TOKEN<u64 , vector<bool>>>"),
            format!("{}::pair::Pair<{}::sui::SUI, {}::token::TOKEN<u64, vector<bool>>>", address("1"), address("2"), address("abc"))
        );
        assert_eq!(displayed(&format!("0x2::{0}::{0}", "a".repeat(128))), format!("{}::{1}::{1}", address("2"), "a".repeat(128)));

        let invalid = [
            "",
            "u8 ",
            " u8",
            "u7",
            "U64",
            "vector<u8",
            "vector<>",
            "vector< u8>",
            "0x2::sui",
            "2::sui::SUI",
            "0X2::sui::SUI",
            "0x::sui::SUI",
            "0x2::sui::SUI<>",
            "0x2::sui::SUI<u8,>",
            "0x2::sui::SUI<u8>>",
            "0x2::1sui::SUI",
            "0x2::_::SUI",
            "0xg::sui::SUI",
            "0x2:sui::SUI",
            &format!("0x{}::sui::SUI", "1".repeat(65)),
            &format!("0x2::sui::{}", "a".repeat(129)),
        ];
        for value in invalid {
            assert!(value.parse::<TypeTag>().is_err(), "{value}");
        }
    }

    #[test]
    fn test_type_tag_limits() {
        for (depth, valid) in [(MAX_TYPE_TAG_DEPTH - 1, true), (MAX_TYPE_TAG_DEPTH, false), (400, false)] {
            let text = format!("{}u8{}", VECTOR_PREFIX.repeat(depth), ">".repeat(depth));
            let bytes = [vec![6; depth], vec![1]].concat();

            assert_eq!(text.parse::<TypeTag>().is_ok(), valid, "{depth}");
            assert_eq!(bcs::from_bytes::<TypeTag>(&bytes).is_ok(), valid, "{depth}");
        }
        for (count, valid) in [(MAX_TYPE_TAG_NODES - 1, true), (MAX_TYPE_TAG_NODES, false)] {
            let text = format!("0x2::m::T<{}>", vec!["u8"; count].join(","));
            let bytes = [vec![7], vec![0; 31], b"\x02\x01m\x01T".to_vec(), bcs::to_bytes(&vec![TypeTag::U8; count]).unwrap()].concat();

            assert_eq!(text.parse::<TypeTag>().is_ok(), valid, "{count}");
            assert_eq!(bcs::from_bytes::<TypeTag>(&bytes).is_ok(), valid, "{count}");
        }
    }

    #[test]
    fn test_type_tag_bcs() {
        let type_tag: TypeTag = "vector<0x2::coin::Coin<0x2::sui::SUI>>".parse().unwrap();
        let bytes = bcs::to_bytes(&type_tag).unwrap();
        let framework = [vec![0; 31], vec![2]].concat();

        assert_eq!(bytes, [vec![6, 7], framework.clone(), b"\x04coin\x04Coin\x01\x07".to_vec(), framework, b"\x03sui\x03SUI\x00".to_vec()].concat());
        assert_eq!(bcs::from_bytes::<TypeTag>(&bytes).unwrap(), type_tag);
        assert_eq!(
            [TypeTag::Bool, TypeTag::U8, TypeTag::U64, TypeTag::U128, TypeTag::Address, TypeTag::Signer, TypeTag::U16, TypeTag::U32, TypeTag::U256].map(|type_tag| bcs::to_bytes(&type_tag).unwrap()),
            [[0], [1], [2], [3], [4], [5], [8], [9], [10]].map(Vec::from)
        );
        assert!(bcs::from_bytes::<Identifier>(b"\x041sui").is_err());
        assert!(bcs::from_bytes::<TypeTag>(&[11]).is_err());
    }

    #[test]
    fn test_identifier_new() {
        for valid in ["a", "A1_", "_a", "__", &"a".repeat(MAX_IDENTIFIER_LENGTH)] {
            assert_eq!(Identifier::new(valid).unwrap().to_string(), valid);
        }
        for invalid in ["", "_", "1a", "a-b", "a b", "é", &"a".repeat(MAX_IDENTIFIER_LENGTH + 1)] {
            assert!(Identifier::new(invalid).is_err(), "{invalid}");
        }
    }
}
