use std::borrow::Cow;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::error::ParseError;
use crate::percent::{self, CONTROLS};

const PUNYCODE_PREFIX: &str = "xn--";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host<'a> {
    Domain(&'a str),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostInternal {
    None,
    Domain,
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

pub(crate) enum ParsedHost<'a> {
    Domain(Cow<'a, str>),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

enum Ipv4Number {
    Value(u32),
    Overflow,
    Invalid,
}

impl<'a> ParsedHost<'a> {
    pub(crate) fn parse(input: &'a str) -> Result<Self, ParseError> {
        if let Some(address) = input.strip_prefix('[') {
            return Ok(Self::Ipv6(parse_ipv6(address.strip_suffix(']').ok_or(ParseError::InvalidIpv6Address)?)?));
        }
        let decoded = percent::decode(input.as_bytes());
        if !decoded.is_ascii() || decoded.iter().any(|byte| is_forbidden_domain_byte(*byte)) {
            return Err(ParseError::IdnaError);
        }
        let domain = match decoded {
            Cow::Borrowed(_) if !input.bytes().any(|byte| byte.is_ascii_uppercase()) => Cow::Borrowed(input),
            decoded => Cow::Owned(decoded.iter().map(|byte| char::from(byte.to_ascii_lowercase())).collect()),
        };
        if domain.split('.').any(|label| label.starts_with(PUNYCODE_PREFIX)) {
            return Err(ParseError::IdnaError);
        }
        if domain.is_empty() {
            return Err(ParseError::EmptyHost);
        }
        match ends_in_a_number(&domain) {
            true => Ok(Self::Ipv4(parse_ipv4(&domain)?)),
            false => Ok(Self::Domain(domain)),
        }
    }

    pub(crate) fn parse_opaque(input: &'a str) -> Result<Self, ParseError> {
        if let Some(address) = input.strip_prefix('[') {
            return Ok(Self::Ipv6(parse_ipv6(address.strip_suffix(']').ok_or(ParseError::InvalidIpv6Address)?)?));
        }
        if input.bytes().any(is_forbidden_host_byte) {
            return Err(ParseError::InvalidDomainCharacter);
        }
        match input.bytes().any(|byte| CONTROLS.contains(byte)) {
            true => {
                let mut domain = String::with_capacity(input.len());
                percent::encode_into(&mut domain, input, CONTROLS);
                Ok(Self::Domain(Cow::Owned(domain)))
            }
            false => Ok(Self::Domain(Cow::Borrowed(input))),
        }
    }

    pub(crate) fn serialize_into(&self, output: &mut String) {
        match self {
            Self::Domain(domain) => output.push_str(domain),
            Self::Ipv4(address) => output.push_str(&address.to_string()),
            Self::Ipv6(address) => {
                output.push('[');
                push_ipv6(output, address);
                output.push(']');
            }
        }
    }

    pub(crate) fn internal(&self) -> HostInternal {
        match self {
            Self::Domain(domain) if domain.is_empty() => HostInternal::None,
            Self::Domain(_) => HostInternal::Domain,
            Self::Ipv4(address) => HostInternal::Ipv4(*address),
            Self::Ipv6(address) => HostInternal::Ipv6(*address),
        }
    }
}

fn is_forbidden_domain_byte(byte: u8) -> bool {
    byte <= b' ' || byte == 0x7F || b"%#/:<>?@[\\]^|".contains(&byte)
}

fn is_forbidden_host_byte(byte: u8) -> bool {
    b"\0\t\n\r #/:<>?@[\\]^|".contains(&byte)
}

fn ends_in_a_number(domain: &str) -> bool {
    let mut labels = domain.rsplit('.');
    let last = match (labels.next(), labels.next()) {
        (Some(""), Some(previous)) => previous,
        (Some(""), None) | (None, _) => return false,
        (Some(last), _) => last,
    };
    if !last.is_empty() && last.bytes().all(|byte| byte.is_ascii_digit()) {
        return true;
    }
    match ipv4_number(last) {
        Ipv4Number::Value(_) | Ipv4Number::Overflow => true,
        Ipv4Number::Invalid => false,
    }
}

fn ipv4_number(input: &str) -> Ipv4Number {
    if input.is_empty() {
        return Ipv4Number::Invalid;
    }
    let (digits, radix) = match input.strip_prefix("0x").or_else(|| input.strip_prefix("0X")) {
        Some(digits) => (digits, 16),
        None if input.len() >= 2 && input.starts_with('0') => (&input[1..], 8),
        None => (input, 10),
    };
    if digits.is_empty() {
        return Ipv4Number::Value(0);
    }
    if !digits.chars().all(|character| character.is_digit(radix)) {
        return Ipv4Number::Invalid;
    }
    match u32::from_str_radix(digits, radix) {
        Ok(value) => Ipv4Number::Value(value),
        Err(_) => Ipv4Number::Overflow,
    }
}

fn parse_ipv4(domain: &str) -> Result<Ipv4Addr, ParseError> {
    let domain = domain.strip_suffix('.').unwrap_or(domain);
    let mut numbers = [0u32; 4];
    let mut count = 0;
    for part in domain.split('.') {
        let slot = numbers.get_mut(count).ok_or(ParseError::InvalidIpv4Address)?;
        *slot = match ipv4_number(part) {
            Ipv4Number::Value(value) => value,
            Ipv4Number::Overflow | Ipv4Number::Invalid => return Err(ParseError::InvalidIpv4Address),
        };
        count += 1;
    }
    let (last, leading) = numbers[..count].split_last().ok_or(ParseError::InvalidIpv4Address)?;
    if leading.iter().any(|number| *number > 255) || *last > u32::MAX >> (8 * leading.len()) {
        return Err(ParseError::InvalidIpv4Address);
    }
    let address = leading.iter().enumerate().fold(*last, |address, (index, number)| address | number << (8 * (3 - index)));
    Ok(Ipv4Addr::from(address))
}

fn parse_ipv6(input: &str) -> Result<Ipv6Addr, ParseError> {
    let input = input.as_bytes();
    let length = input.len();
    let mut pieces = [0u16; 8];
    let mut piece_pointer = 0;
    let mut compress_pointer = None;
    let mut index = 0;
    let mut is_ipv4 = false;

    if length < 2 {
        return Err(ParseError::InvalidIpv6Address);
    }
    if input[0] == b':' {
        if input[1] != b':' {
            return Err(ParseError::InvalidIpv6Address);
        }
        index = 2;
        piece_pointer = 1;
        compress_pointer = Some(1);
    }

    while index < length {
        if piece_pointer == 8 {
            return Err(ParseError::InvalidIpv6Address);
        }
        if input[index] == b':' {
            if compress_pointer.is_some() {
                return Err(ParseError::InvalidIpv6Address);
            }
            index += 1;
            piece_pointer += 1;
            compress_pointer = Some(piece_pointer);
            continue;
        }
        let start = index;
        let end = length.min(start + 4);
        let mut value = 0u16;
        while index < end {
            match char::from(input[index]).to_digit(16) {
                Some(digit) => {
                    value = value * 0x10 + digit as u16;
                    index += 1;
                }
                None => break,
            }
        }
        if index < length {
            match input[index] {
                b'.' => {
                    if index == start || piece_pointer > 6 {
                        return Err(ParseError::InvalidIpv6Address);
                    }
                    index = start;
                    is_ipv4 = true;
                    break;
                }
                b':' => {
                    index += 1;
                    if index == length {
                        return Err(ParseError::InvalidIpv6Address);
                    }
                }
                _ => return Err(ParseError::InvalidIpv6Address),
            }
        }
        pieces[piece_pointer] = value;
        piece_pointer += 1;
    }

    if is_ipv4 {
        let mut numbers_seen = 0;
        while index < length {
            if numbers_seen > 0 {
                if numbers_seen < 4 && input[index] == b'.' {
                    index += 1;
                } else {
                    return Err(ParseError::InvalidIpv6Address);
                }
            }
            let mut ipv4_piece: Option<u16> = None;
            while let Some(digit) = input.get(index).filter(|byte| byte.is_ascii_digit()).map(|byte| u16::from(byte - b'0')) {
                ipv4_piece = match ipv4_piece {
                    None => Some(digit),
                    Some(0) => return Err(ParseError::InvalidIpv6Address),
                    Some(value) if value * 10 + digit > 255 => return Err(ParseError::InvalidIpv6Address),
                    Some(value) => Some(value * 10 + digit),
                };
                index += 1;
            }
            let piece = pieces.get_mut(piece_pointer).ok_or(ParseError::InvalidIpv6Address)?;
            *piece = *piece * 0x100 + ipv4_piece.ok_or(ParseError::InvalidIpv6Address)?;
            numbers_seen += 1;
            if numbers_seen == 2 || numbers_seen == 4 {
                piece_pointer += 1;
            }
        }
        if numbers_seen != 4 {
            return Err(ParseError::InvalidIpv6Address);
        }
    }

    if index < length {
        return Err(ParseError::InvalidIpv6Address);
    }

    match compress_pointer {
        Some(compress_pointer) => {
            let mut swaps = piece_pointer - compress_pointer;
            piece_pointer = 7;
            while swaps > 0 {
                pieces.swap(piece_pointer, compress_pointer + swaps - 1);
                swaps -= 1;
                piece_pointer -= 1;
            }
        }
        None if piece_pointer != 8 => return Err(ParseError::InvalidIpv6Address),
        None => {}
    }
    Ok(Ipv6Addr::from(pieces))
}

fn push_ipv6(output: &mut String, address: &Ipv6Addr) {
    let segments = address.segments();
    let compressed = longest_zero_sequence(&segments);
    let mut index = 0;
    while index < segments.len() {
        if let Some((start, end)) = compressed
            && index == start
        {
            output.push_str(if index == 0 { "::" } else { ":" });
            index = end;
            if index == segments.len() {
                break;
            }
        }
        output.push_str(&format!("{:x}", segments[index]));
        if index < segments.len() - 1 {
            output.push(':');
        }
        index += 1;
    }
}

fn longest_zero_sequence(segments: &[u16; 8]) -> Option<(usize, usize)> {
    let mut longest: Option<(usize, usize)> = None;
    let mut start = None;
    for index in 0..=segments.len() {
        match (segments.get(index), start) {
            (Some(0), None) => start = Some(index),
            (Some(0), Some(_)) => {}
            (_, Some(sequence_start)) => {
                if longest.is_none_or(|(longest_start, longest_end)| index - sequence_start > longest_end - longest_start) {
                    longest = Some((sequence_start, index));
                }
                start = None;
            }
            (_, None) => {}
        }
    }
    longest.filter(|(start, end)| end - start >= 2)
}
