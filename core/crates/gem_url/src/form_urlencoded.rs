use std::borrow::Cow;

use crate::percent;

pub fn parse(input: &[u8]) -> Parse<'_> {
    Parse { input }
}

pub fn byte_serialize(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len());
    push_serialized(&mut output, input);
    output
}

pub fn serialize<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut output = String::new();
    for (name, value) in pairs {
        if !output.is_empty() {
            output.push('&');
        }
        push_serialized(&mut output, name.as_bytes());
        output.push('=');
        push_serialized(&mut output, value.as_bytes());
    }
    output
}

#[derive(Debug, Clone, Copy)]
pub struct Parse<'a> {
    input: &'a [u8],
}

impl<'a> Iterator for Parse<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.input.is_empty() {
            let (sequence, rest) = split_once(self.input, b'&');
            self.input = rest;
            if !sequence.is_empty() {
                let (name, value) = split_once(sequence, b'=');
                return Some((decode(name), decode(value)));
            }
        }
        None
    }
}

fn split_once(input: &[u8], separator: u8) -> (&[u8], &[u8]) {
    match input.iter().position(|byte| *byte == separator) {
        Some(index) => (&input[..index], &input[index + 1..]),
        None => (input, &[]),
    }
}

fn decode(input: &[u8]) -> Cow<'_, str> {
    let decoded = match input.contains(&b'+') {
        true => Cow::Owned(percent::decode(&input.iter().map(|byte| if *byte == b'+' { b' ' } else { *byte }).collect::<Vec<_>>()).into_owned()),
        false => percent::decode(input),
    };
    match decoded {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8(bytes).unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())),
    }
}

fn push_serialized(output: &mut String, input: &[u8]) {
    for byte in input {
        match byte {
            b'*' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'_' | b'a'..=b'z' => output.push(char::from(*byte)),
            b' ' => output.push('+'),
            _ => percent::push_encoded_byte(output, *byte),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse(b"foo=%C3%A9%26&bar=&foo=%23").collect::<Vec<_>>(),
            vec![("foo".into(), "é&".into()), ("bar".into(), "".into()), ("foo".into(), "#".into())]
        );
        assert_eq!(
            parse(b"a+b=c%20d+e&&=f&g&%zz=%FF").collect::<Vec<_>>(),
            vec![("a b".into(), "c d e".into()), ("".into(), "f".into()), ("g".into(), "".into()), ("%zz".into(), "\u{FFFD}".into())]
        );
        assert_eq!(parse(b"").count(), 0);
    }

    #[test]
    fn test_byte_serialize() {
        assert_eq!(byte_serialize("é& *-._~+/".as_bytes()), "%C3%A9%26+*-._%7E%2B%2F");
        assert_eq!(byte_serialize(b"0x6331c6eded7cfe4ed578e41a57855102b3fd60b3"), "0x6331c6eded7cfe4ed578e41a57855102b3fd60b3");
    }

    #[test]
    fn test_serialize() {
        assert_eq!(serialize([("foo", "é&"), ("bar", ""), ("foo", "#")]), "foo=%C3%A9%26&bar=&foo=%23");
        assert_eq!(serialize([]), "");
    }
}
