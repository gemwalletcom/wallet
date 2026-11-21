#![no_main]

use gem_url::{Host, ParseError, Url, form_urlencoded};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, PartialEq)]
struct Components {
    serialization: String,
    scheme: String,
    username: String,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    known_port: Option<u16>,
    path: String,
    segments: Option<Vec<String>>,
    query: Option<String>,
    fragment: Option<String>,
    pairs: Vec<(String, String)>,
}

fn components(url: &Url) -> Components {
    Components {
        serialization: url.as_str().to_string(),
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        password: url.password().map(str::to_string),
        host: url.host().map(|host| match host {
            Host::Domain(domain) => format!("domain {domain}"),
            Host::Ipv4(address) => format!("ipv4 {address}"),
            Host::Ipv6(address) => format!("ipv6 {address}"),
        }),
        port: url.port(),
        known_port: url.port_or_known_default(),
        path: url.path().to_string(),
        segments: url.path_segments().map(|segments| segments.map(str::to_string).collect()),
        query: url.query().map(str::to_string),
        fragment: url.fragment().map(str::to_string),
        pairs: url.query_pairs().map(|(name, value)| (name.into_owned(), value.into_owned())).collect(),
    }
}

fn oracle_components(url: &url::Url) -> Components {
    Components {
        serialization: url.as_str().to_string(),
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        password: url.password().map(str::to_string),
        host: url.host().map(|host| match host {
            url::Host::Domain(domain) => format!("domain {domain}"),
            url::Host::Ipv4(address) => format!("ipv4 {address}"),
            url::Host::Ipv6(address) => format!("ipv6 {address}"),
        }),
        port: url.port(),
        known_port: url.port_or_known_default(),
        path: url.path().to_string(),
        segments: url.path_segments().map(|segments| segments.map(str::to_string).collect()),
        query: url.query().map(str::to_string),
        fragment: url.fragment().map(str::to_string),
        pairs: url.query_pairs().map(|(name, value)| (name.into_owned(), value.into_owned())).collect(),
    }
}

fn raw_host(input: &str) -> Option<String> {
    let input = input.trim_matches(|character: char| character <= ' ').replace(['\t', '\n', '\r'], "");
    let (_, rest) = input.split_once(':')?;
    let rest = rest.trim_start_matches(['/', '\\']);
    let authority = &rest[..rest.find(['/', '\\', '?', '#']).unwrap_or(rest.len())];
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let mut inside_brackets = false;
    let end = host
        .char_indices()
        .find(|(_, character)| {
            inside_brackets = (inside_brackets || *character == '[') && *character != ']';
            *character == ':' && !inside_brackets
        })
        .map_or(host.len(), |(index, _)| index);
    Some(host[..end].to_string())
}

fn needs_idna(input: &str) -> bool {
    raw_host(input).is_some_and(|host| {
        let decoded: Vec<u8> = percent_encoding::percent_decode(host.as_bytes()).collect();
        !decoded.is_ascii() || decoded.to_ascii_lowercase().split(|byte| *byte == b'.').any(|label| label.starts_with(b"xn--"))
    })
}

fuzz_target!(|input: &str| {
    match (Url::parse(input), url::Url::parse(input)) {
        (Ok(url), Ok(oracle)) => {
            assert_eq!(components(&url), oracle_components(&oracle));
            let origin = url.origin().ascii_serialization();
            let inner_scheme = url::Url::parse(oracle.path()).map(|inner| inner.scheme().to_string()).unwrap_or_default();
            match (oracle.scheme(), inner_scheme.as_str()) {
                ("blob", "http" | "https") => assert!(origin == oracle.origin().ascii_serialization() || needs_idna(oracle.path()), "{origin}"),
                ("blob", _) => assert_eq!(origin, "null"),
                _ => assert_eq!(origin, oracle.origin().ascii_serialization()),
            }
            let reparsed = Url::parse(url.as_str()).expect("a serialized URL parses");
            assert_eq!(components(&reparsed), components(&url));
            assert_eq!(url.origin() == reparsed.origin(), origin != "null");
        }
        (Err(ParseError::IdnaError), Ok(oracle)) => assert!(needs_idna(input), "{:?}", oracle.as_str()),
        (Err(ParseError::UnsupportedScheme), Ok(oracle)) => assert_eq!(oracle.scheme(), "file"),
        (Err(ParseError::IdnaError | ParseError::UnsupportedScheme), Err(_)) => {}
        (Err(error), Err(oracle)) => assert_eq!(error.to_string(), oracle.to_string()),
        (Err(error), Ok(oracle)) => panic!("{error} {:?}", oracle.as_str()),
        (Ok(url), Err(oracle)) => panic!("{oracle} {:?}", url.as_str()),
    }

    let pairs: Vec<(String, String)> = form_urlencoded::parse(input.as_bytes()).map(|(name, value)| (name.into_owned(), value.into_owned())).collect();
    let oracle_pairs: Vec<(String, String)> = ::url::form_urlencoded::parse(input.as_bytes()).map(|(name, value)| (name.into_owned(), value.into_owned())).collect();
    assert_eq!(pairs, oracle_pairs);
    assert_eq!(form_urlencoded::byte_serialize(input.as_bytes()), ::url::form_urlencoded::byte_serialize(input.as_bytes()).collect::<String>());
    let serialized = form_urlencoded::serialize(pairs.iter().map(|(name, value)| (name.as_str(), value.as_str())));
    assert_eq!(serialized, ::url::form_urlencoded::Serializer::new(String::new()).extend_pairs(&pairs).finish());
});
