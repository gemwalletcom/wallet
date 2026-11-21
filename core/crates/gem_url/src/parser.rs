use std::borrow::Cow;

use crate::error::ParseError;
use crate::host::{HostInternal, ParsedHost};
use crate::percent::{self, CONTROLS, EncodeSet, FRAGMENT, PATH, QUERY, SPECIAL_QUERY, USERINFO};
use crate::url::Url;

const IGNORED_CHARACTERS: [char; 3] = ['\t', '\n', '\r'];
const SINGLE_DOT_SEGMENTS: [&str; 2] = [".", "%2e"];
const DOUBLE_DOT_SEGMENTS: [&str; 4] = ["..", ".%2e", "%2e.", "%2e%2e"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchemeType {
    Special,
    File,
    NotSpecial,
}

impl SchemeType {
    pub(crate) fn from_scheme(scheme: &str) -> Self {
        match scheme {
            "http" | "https" | "ws" | "wss" | "ftp" => Self::Special,
            "file" => Self::File,
            _ => Self::NotSpecial,
        }
    }

    fn is_special(self) -> bool {
        match self {
            Self::Special | Self::File => true,
            Self::NotSpecial => false,
        }
    }

    fn query_set(self) -> EncodeSet {
        match self.is_special() {
            true => SPECIAL_QUERY,
            false => QUERY,
        }
    }
}

pub(crate) fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        _ => None,
    }
}

pub(crate) fn parse(input: &str) -> Result<Url, ParseError> {
    let input = input.trim_matches(|character: char| character <= ' ');
    let input = match input.contains(IGNORED_CHARACTERS) {
        true => Cow::Owned(input.replace(IGNORED_CHARACTERS, "")),
        false => Cow::Borrowed(input),
    };
    let (scheme, rest) = split_scheme(&input).ok_or(ParseError::RelativeUrlWithoutBase)?;
    let mut parser = Parser {
        serialization: String::with_capacity(input.len() + 1),
    };
    parser.serialization.extend(scheme.chars().map(|character| character.to_ascii_lowercase()));
    let scheme_end = parser.serialization.len();
    let scheme_type = SchemeType::from_scheme(&parser.serialization);
    parser.serialization.push(':');
    match scheme_type {
        SchemeType::Special => parser.after_double_slash(rest.trim_start_matches(['/', '\\']), scheme_type, scheme_end),
        SchemeType::File => Err(ParseError::UnsupportedScheme),
        SchemeType::NotSpecial => match rest.strip_prefix("//") {
            Some(rest) => parser.after_double_slash(rest, scheme_type, scheme_end),
            None => Ok(parser.without_authority(rest, scheme_end)),
        },
    }
}

fn split_scheme(input: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = input.split_once(':')?;
    let mut bytes = scheme.bytes();
    (bytes.next()?.is_ascii_alphabetic() && bytes.all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte))).then_some((scheme, rest))
}

struct Parser {
    serialization: String,
}

struct Authority {
    username_end: usize,
    host_start: usize,
    host_end: usize,
    host: HostInternal,
    port: Option<u16>,
}

impl Parser {
    fn without_authority(mut self, input: &str, scheme_end: usize) -> Url {
        let path_start = self.serialization.len();
        let rest = match input.strip_prefix('/') {
            Some(rest) => {
                self.serialization.push('/');
                self.parse_path(SchemeType::NotSpecial, path_start, rest)
            }
            None => self.parse_opaque_path(input),
        };
        let authority = Authority {
            username_end: path_start,
            host_start: path_start,
            host_end: path_start,
            host: HostInternal::None,
            port: None,
        };
        self.finish(SchemeType::NotSpecial, scheme_end, authority, path_start, rest)
    }

    fn after_double_slash(mut self, input: &str, scheme_type: SchemeType, scheme_end: usize) -> Result<Url, ParseError> {
        self.serialization.push_str("//");
        let authority_start = self.serialization.len();
        let (username_end, rest) = self.parse_userinfo(input, scheme_type)?;
        let has_userinfo = authority_start != self.serialization.len();
        let host_start = self.serialization.len();
        let (host, rest) = parse_host(rest, scheme_type)?;
        host.serialize_into(&mut self.serialization);
        let host_end = self.serialization.len();
        if let ParsedHost::Domain(domain) = &host
            && domain.is_empty()
            && (rest.starts_with(':') || scheme_type.is_special())
        {
            return Err(ParseError::EmptyHost);
        }
        let (port, rest) = match rest.strip_prefix(':') {
            Some(rest) => parse_port(rest, default_port(&self.serialization[..scheme_end]))?,
            None => (None, rest),
        };
        if let Some(port) = port {
            self.serialization.push(':');
            self.serialization.push_str(&port.to_string());
        }
        let host = host.internal();
        if host == HostInternal::None && has_userinfo {
            return Err(ParseError::EmptyHost);
        }
        let path_start = self.serialization.len();
        let rest = self.parse_path_start(scheme_type, rest);
        let authority = Authority {
            username_end,
            host_start,
            host_end,
            host,
            port,
        };
        Ok(self.finish(scheme_type, scheme_end, authority, path_start, rest))
    }

    fn parse_userinfo<'i>(&mut self, input: &'i str, scheme_type: SchemeType) -> Result<(usize, &'i str), ParseError> {
        let authority_end = input.bytes().position(|byte| is_delimiter(byte, scheme_type)).unwrap_or(input.len());
        let Some(at) = input[..authority_end].rfind('@') else {
            return Ok((self.serialization.len(), input));
        };
        let rest = &input[at + 1..];
        if at == 0 {
            if rest.bytes().next().is_some_and(|byte| is_delimiter(byte, scheme_type)) {
                return Err(ParseError::EmptyHost);
            }
            return Ok((self.serialization.len(), rest));
        }
        let (username, password) = input[..at].split_once(':').unwrap_or((&input[..at], ""));
        percent::encode_into(&mut self.serialization, username, USERINFO);
        let username_end = self.serialization.len();
        if !password.is_empty() {
            self.serialization.push(':');
            percent::encode_into(&mut self.serialization, password, USERINFO);
        }
        if !username.is_empty() || !password.is_empty() {
            self.serialization.push('@');
        }
        Ok((username_end, rest))
    }

    fn parse_path_start<'i>(&mut self, scheme_type: SchemeType, input: &'i str) -> &'i str {
        let path_start = self.serialization.len();
        match (scheme_type.is_special(), input.as_bytes().first()) {
            (true, _) => {
                self.serialization.push('/');
                self.parse_path(scheme_type, path_start, input.strip_prefix(['/', '\\']).unwrap_or(input))
            }
            (false, Some(b'?' | b'#')) => input,
            (false, Some(b'/') | None) => self.parse_path(scheme_type, path_start, input),
            (false, Some(_)) => {
                self.serialization.push('/');
                self.parse_path(scheme_type, path_start, input)
            }
        }
    }

    fn parse_path<'i>(&mut self, scheme_type: SchemeType, path_start: usize, input: &'i str) -> &'i str {
        let mut rest = input;
        loop {
            let segment_start = self.serialization.len();
            let end = rest.bytes().position(|byte| is_delimiter(byte, scheme_type)).unwrap_or(rest.len());
            percent::encode_into(&mut self.serialization, &rest[..end], PATH);
            let ends_with_slash = rest[end..].starts_with(['/', '\\']);
            rest = match ends_with_slash {
                true => {
                    self.serialization.push('/');
                    &rest[end + 1..]
                }
                false => &rest[end..],
            };
            let segment = &self.serialization[segment_start..self.serialization.len() - usize::from(ends_with_slash)];
            let is_double_dot = DOUBLE_DOT_SEGMENTS.iter().any(|dots| segment.eq_ignore_ascii_case(dots));
            let is_single_dot = SINGLE_DOT_SEGMENTS.iter().any(|dot| segment.eq_ignore_ascii_case(dot));
            if is_double_dot {
                self.serialization.truncate(segment_start);
                if self.serialization.ends_with('/') && last_slash_can_be_removed(&self.serialization, path_start) {
                    self.serialization.pop();
                }
                self.shorten_path(path_start);
                if ends_with_slash && !self.serialization.ends_with('/') {
                    self.serialization.push('/');
                }
            } else if is_single_dot {
                self.serialization.truncate(segment_start);
                if !self.serialization.ends_with('/') {
                    self.serialization.push('/');
                }
            }
            if !ends_with_slash {
                return rest;
            }
        }
    }

    fn shorten_path(&mut self, path_start: usize) {
        if let Some(slash) = self.serialization.get(path_start..).and_then(|path| path.rfind('/')) {
            self.serialization.truncate(path_start + slash + 1);
        }
    }

    fn parse_opaque_path<'i>(&mut self, input: &'i str) -> &'i str {
        let end = input.find(['?', '#']).unwrap_or(input.len());
        percent::encode_into(&mut self.serialization, &input[..end], CONTROLS);
        &input[end..]
    }

    fn finish(mut self, scheme_type: SchemeType, scheme_end: usize, authority: Authority, path_start: usize, rest: &str) -> Url {
        let path_start = match path_start == scheme_end + 1 && self.serialization[path_start..].starts_with("//") {
            true => {
                self.serialization.insert_str(path_start, "/.");
                path_start + 2
            }
            false => path_start,
        };
        let (before_fragment, fragment) = match rest.split_once('#') {
            Some((before_fragment, fragment)) => (before_fragment, Some(fragment)),
            None => (rest, None),
        };
        let query_start = before_fragment.strip_prefix('?').map(|query| {
            let start = self.serialization.len();
            self.serialization.push('?');
            percent::encode_into(&mut self.serialization, query, scheme_type.query_set());
            start
        });
        let fragment_start = fragment.map(|fragment| {
            let start = self.serialization.len();
            self.serialization.push('#');
            percent::encode_into(&mut self.serialization, fragment, FRAGMENT);
            start
        });
        Url {
            serialization: self.serialization,
            scheme_end,
            username_end: authority.username_end,
            host_start: authority.host_start,
            host_end: authority.host_end,
            host: authority.host,
            port: authority.port,
            path_start,
            query_start,
            fragment_start,
        }
    }
}

fn is_delimiter(byte: u8, scheme_type: SchemeType) -> bool {
    match byte {
        b'/' | b'?' | b'#' => true,
        b'\\' => scheme_type.is_special(),
        _ => false,
    }
}

fn parse_host(input: &str, scheme_type: SchemeType) -> Result<(ParsedHost<'_>, &str), ParseError> {
    let mut inside_brackets = false;
    let end = input
        .bytes()
        .position(|byte| match byte {
            b':' => !inside_brackets,
            b'[' => {
                inside_brackets = true;
                false
            }
            b']' => {
                inside_brackets = false;
                false
            }
            _ => is_delimiter(byte, scheme_type),
        })
        .unwrap_or(input.len());
    let (host, rest) = input.split_at(end);
    let host = match (scheme_type.is_special(), host.is_empty()) {
        (true, true) => return Err(ParseError::EmptyHost),
        (true, false) => ParsedHost::parse(host)?,
        (false, _) => ParsedHost::parse_opaque(host)?,
    };
    Ok((host, rest))
}

fn parse_port(input: &str, default_port: Option<u16>) -> Result<(Option<u16>, &str), ParseError> {
    let end = input.find(['/', '\\', '?', '#']).unwrap_or(input.len());
    let (digits, rest) = input.split_at(end);
    let port = digits
        .bytes()
        .try_fold(0u16, |port, byte| match byte.is_ascii_digit() {
            true => port.checked_mul(10)?.checked_add(u16::from(byte - b'0')),
            false => None,
        })
        .ok_or(ParseError::InvalidPort)?;
    Ok(((!digits.is_empty() && Some(port) != default_port).then_some(port), rest))
}

fn last_slash_can_be_removed(serialization: &str, path_start: usize) -> bool {
    serialization[path_start..serialization.len() - 1]
        .rfind('/')
        .is_some_and(|start| !path_starts_with_windows_drive_letter(&serialization.as_bytes()[path_start + start..]))
}

fn path_starts_with_windows_drive_letter(path: &[u8]) -> bool {
    match path {
        [b'/' | b'\\' | b'?' | b'#', letter, b':' | b'|'] | [b'/' | b'\\' | b'?' | b'#', letter, b':' | b'|', b'/' | b'\\' | b'?' | b'#', ..] => letter.is_ascii_alphabetic(),
        _ => false,
    }
}
