use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::Split;

use crate::error::ParseError;
use crate::form_urlencoded::{self, Parse};
use crate::host::{Host, HostInternal};
use crate::parser::{self, SchemeType};

const BLOB_SCHEME: &str = "blob";
const OPAQUE_ORIGIN: &str = "null";
const AUTHORITY_PREFIX: &str = "://";

#[derive(Debug, Clone)]
pub struct Url {
    pub(crate) serialization: String,
    pub(crate) scheme_end: usize,
    pub(crate) username_end: usize,
    pub(crate) host_start: usize,
    pub(crate) host_end: usize,
    pub(crate) host: HostInternal,
    pub(crate) port: Option<u16>,
    pub(crate) path_start: usize,
    pub(crate) query_start: Option<usize>,
    pub(crate) fragment_start: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Origin {
    Opaque,
    Tuple(String, String, u16),
}

impl Url {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parser::parse(input)
    }

    pub fn as_str(&self) -> &str {
        &self.serialization
    }

    pub fn scheme(&self) -> &str {
        &self.serialization[..self.scheme_end]
    }

    pub fn username(&self) -> &str {
        match self.has_authority() && self.username_end > self.scheme_end + AUTHORITY_PREFIX.len() {
            true => &self.serialization[self.scheme_end + AUTHORITY_PREFIX.len()..self.username_end],
            false => "",
        }
    }

    pub fn password(&self) -> Option<&str> {
        match self.has_authority() && self.serialization.as_bytes().get(self.username_end) == Some(&b':') {
            true => self.serialization.get(self.username_end + 1..self.host_start - 1),
            false => None,
        }
    }

    pub fn host_str(&self) -> Option<&str> {
        match self.host {
            HostInternal::None => None,
            HostInternal::Domain | HostInternal::Ipv4(_) | HostInternal::Ipv6(_) => Some(&self.serialization[self.host_start..self.host_end]),
        }
    }

    pub fn host(&self) -> Option<Host<'_>> {
        match self.host {
            HostInternal::None => None,
            HostInternal::Domain => Some(Host::Domain(&self.serialization[self.host_start..self.host_end])),
            HostInternal::Ipv4(address) => Some(Host::Ipv4(address)),
            HostInternal::Ipv6(address) => Some(Host::Ipv6(address)),
        }
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn port_or_known_default(&self) -> Option<u16> {
        self.port.or_else(|| parser::default_port(self.scheme()))
    }

    pub fn path(&self) -> &str {
        match self.query_start.or(self.fragment_start) {
            Some(end) => &self.serialization[self.path_start..end],
            None => &self.serialization[self.path_start..],
        }
    }

    pub fn path_segments(&self) -> Option<Split<'_, char>> {
        self.path().strip_prefix('/').map(|path| path.split('/'))
    }

    pub fn query(&self) -> Option<&str> {
        let start = self.query_start? + 1;
        match self.fragment_start {
            Some(end) => Some(&self.serialization[start..end]),
            None => Some(&self.serialization[start..]),
        }
    }

    pub fn query_pairs(&self) -> Parse<'_> {
        form_urlencoded::parse(self.query().unwrap_or_default().as_bytes())
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment_start.map(|start| &self.serialization[start + 1..])
    }

    pub fn origin(&self) -> Origin {
        match SchemeType::from_scheme(self.scheme()) {
            SchemeType::Special => match (self.host_str(), self.port_or_known_default()) {
                (Some(host), Some(port)) => Origin::Tuple(self.scheme().to_string(), host.to_string(), port),
                (None, _) | (_, None) => Origin::Opaque,
            },
            SchemeType::File => Origin::Opaque,
            SchemeType::NotSpecial if self.scheme() == BLOB_SCHEME => match Url::parse(self.path()) {
                Ok(inner) if inner.scheme() == "http" || inner.scheme() == "https" => inner.origin(),
                Ok(_) | Err(_) => Origin::Opaque,
            },
            SchemeType::NotSpecial => Origin::Opaque,
        }
    }

    fn has_authority(&self) -> bool {
        self.serialization[self.scheme_end..].starts_with(AUTHORITY_PREFIX)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.serialization)
    }
}

impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.serialization == other.serialization
    }
}

impl Eq for Url {}

impl Hash for Url {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.serialization.hash(state);
    }
}

impl Origin {
    pub fn ascii_serialization(&self) -> String {
        match self {
            Self::Opaque => OPAQUE_ORIGIN.to_string(),
            Self::Tuple(scheme, host, port) if parser::default_port(scheme) == Some(*port) => format!("{scheme}://{host}"),
            Self::Tuple(scheme, host, port) => format!("{scheme}://{host}:{port}"),
        }
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tuple(scheme, host, port), Self::Tuple(other_scheme, other_host, other_port)) => scheme == other_scheme && host == other_host && port == other_port,
            (Self::Opaque, _) | (_, Self::Opaque) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Components {
        href: String,
        origin: Option<String>,
        protocol: String,
        username: String,
        password: String,
        host: String,
        hostname: String,
        port: String,
        pathname: String,
        search: String,
        hash: String,
    }

    #[derive(Debug, Deserialize)]
    struct UrlTest {
        input: String,
        #[serde(flatten)]
        expected: Option<Components>,
    }

    #[test]
    fn test_parse() {
        let tests: Vec<UrlTest> = serde_json::from_str(include_str!("../testdata/urltestdata.json")).unwrap();
        for test in tests {
            let has_origin = test.expected.as_ref().is_some_and(|expected| expected.origin.is_some());
            let actual = Url::parse(&test.input).ok().map(|url| Components {
                href: url.as_str().to_string(),
                origin: has_origin.then(|| url.origin().ascii_serialization()),
                protocol: format!("{}:", url.scheme()),
                username: url.username().to_string(),
                password: url.password().unwrap_or_default().to_string(),
                host: match (url.host_str(), url.port()) {
                    (Some(host), Some(port)) => format!("{host}:{port}"),
                    (host, _) => host.unwrap_or_default().to_string(),
                },
                hostname: url.host_str().unwrap_or_default().to_string(),
                port: url.port().map(|port| port.to_string()).unwrap_or_default(),
                pathname: url.path().to_string(),
                search: url.query().filter(|query| !query.is_empty()).map(|query| format!("?{query}")).unwrap_or_default(),
                hash: url.fragment().filter(|fragment| !fragment.is_empty()).map(|fragment| format!("#{fragment}")).unwrap_or_default(),
            });
            assert_eq!(actual, test.expected, "{:?}", test.input);
        }

        for (input, serialization) in [
            ("http://example.com/", "http://example.com/"),
            ("http://addslash.com", "http://addslash.com/"),
            ("http://@emptyuser.com/", "http://emptyuser.com/"),
            ("http://:@emptypass.com/", "http://emptypass.com/"),
            ("http://user@user.com/", "http://user@user.com/"),
            ("http://user:pass@userpass.com/", "http://user:pass@userpass.com/"),
            ("http://slashquery.com/path/?q=something", "http://slashquery.com/path/?q=something"),
            ("http://noslashquery.com/path?q=something", "http://noslashquery.com/path?q=something"),
            ("  https://GemWallet.com/wc?uri=wc:a@2\n", "https://gemwallet.com/wc?uri=wc:a@2"),
        ] {
            assert_eq!(Url::parse(input).unwrap().as_str(), serialization);
        }

        for input in ["m:/.//\\", "l://\u{1}:"] {
            let url = Url::parse(input).unwrap();
            assert_eq!(Url::parse(url.as_str()).unwrap(), url);
        }

        for input in [
            "http://goșu.ro",
            "http://☃.net/",
            "http://xn--n3h.net/",
            "https://XN--mnchen-3ya.de",
            "https://%E2%98%83.net",
            "https://ｇｅｍｗａｌｌｅｔ.com/wc",
        ] {
            assert_eq!(Url::parse(input), Err(ParseError::IdnaError), "{input}");
        }
        assert_eq!(Url::parse("https://r2---sn-huoa-cvhl.googlevideo.com/crossdomain.xml").unwrap().host_str(), Some("r2---sn-huoa-cvhl.googlevideo.com"));
        assert_eq!(Url::parse("file:///tmp/foo"), Err(ParseError::UnsupportedScheme));
        assert_eq!(Url::parse("example.com"), Err(ParseError::RelativeUrlWithoutBase));
    }

    #[test]
    fn test_host() {
        for (input, host) in [
            ("http://www.mozilla.org", Host::Domain("www.mozilla.org")),
            ("http://1.35.33.49", Host::Ipv4(Ipv4Addr::new(1, 35, 33, 49))),
            ("http://[2001:0db8:85a3:08d3:1319:8a2e:0370:7344]", Host::Ipv6(Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x08d3, 0x1319, 0x8a2e, 0x0370, 0x7344))),
            ("http://[::]", Host::Ipv6(Ipv6Addr::UNSPECIFIED)),
            ("http://[::1]", Host::Ipv6(Ipv6Addr::LOCALHOST)),
            ("http://0x1.0X23.0x21.061", Host::Ipv4(Ipv4Addr::new(1, 35, 33, 49))),
            ("http://0x1232131", Host::Ipv4(Ipv4Addr::new(1, 35, 33, 49))),
            ("http://111", Host::Ipv4(Ipv4Addr::new(0, 0, 0, 111))),
        ] {
            assert_eq!(Url::parse(input).unwrap().host(), Some(host), "{input}");
        }
        for input in ["http://1.35.+33.49", "http://2..2.3", "http://42.0x1232131", "http://192.168.0.257"] {
            assert_eq!(Url::parse(input), Err(ParseError::InvalidIpv4Address), "{input}");
        }
        assert_eq!(Url::parse("http://[0::2]").unwrap().host_str(), Some("[::2]"));
        assert_eq!(Url::parse("http://[0::ffff:0:2]").unwrap().host_str(), Some("[::ffff:0:2]"));
        assert_eq!(Url::parse("mailto:test@example.net").unwrap().host(), None);
    }

    #[test]
    fn test_eq() {
        assert_eq!(Url::parse("https://example.com/").unwrap(), Url::parse("https://example.com:443/").unwrap());
        assert_eq!(Url::parse("http://foo.com").unwrap(), Url::parse("http://foo.com/").unwrap());
        assert_ne!(Url::parse("http://example.com/").unwrap(), Url::parse("http://example.com:8080/").unwrap());
        assert_ne!(Url::parse("http://example.com/").unwrap(), Url::parse("https://example.com/").unwrap());
        assert_ne!(Url::parse("http://foo.com/").unwrap(), Url::parse("http://bar.com/").unwrap());
    }

    #[test]
    fn test_origin() {
        let origin = Url::parse("http://example.net/").unwrap().origin();
        for (input, same) in [
            ("http://example.net:80/", true),
            ("http://example.net:81/", false),
            ("http://example.net", true),
            ("http://example.net/hello", true),
            ("https://example.net", false),
            ("ftp://example.net", false),
            ("http://user@example.net/", true),
            ("http://user:pass@example.net/", true),
            ("blob:http://example.net/", true),
            ("blob:blob:http://example.net/", false),
            ("blob:ws://example.net/", false),
        ] {
            assert_eq!(Url::parse(input).unwrap().origin() == origin, same, "{input}");
        }

        let opaque = Url::parse("blob:malformed//").unwrap().origin();
        assert_eq!(opaque.ascii_serialization(), "null");
        assert_ne!(opaque, opaque.clone());
        assert_eq!(Url::parse("https://example.com:8443/path").unwrap().origin().ascii_serialization(), "https://example.com:8443");
        assert_eq!(Url::parse("https://example.com:443/path").unwrap().origin().ascii_serialization(), "https://example.com");
    }

    #[test]
    fn test_query() {
        let url = Url::parse("https://example.com/products?page=2#fragment").unwrap();
        assert_eq!(url.query(), Some("page=2"));
        assert_eq!(url.fragment(), Some("fragment"));
        assert_eq!(url.query_pairs().collect::<Vec<_>>(), vec![("page".into(), "2".into())]);

        let url = Url::parse("https://example.com/products").unwrap();
        assert_eq!((url.query(), url.fragment(), url.query_pairs().count()), (None, None, 0));

        let url = Url::parse("https://example.com/?country=español").unwrap();
        assert_eq!(url.query(), Some("country=espa%C3%B1ol"));
        assert_eq!(url.query_pairs().collect::<Vec<_>>(), vec![("country".into(), "español".into())]);

        let url = Url::parse("https://example.com/products?page=2&sort=desc").unwrap();
        assert_eq!(url.query_pairs().collect::<Vec<_>>(), vec![("page".into(), "2".into()), ("sort".into(), "desc".into())]);
    }

    #[test]
    fn test_path_segments() {
        let url = Url::parse("mailto:test@example.net").unwrap();
        assert_eq!((url.path(), url.path_segments().map(Iterator::collect::<Vec<_>>)), ("test@example.net", None));

        let url = Url::parse("https://gemwallet.com/zh-cn/tokens/bitcoin/").unwrap();
        assert_eq!(url.path_segments().map(Iterator::collect::<Vec<_>>), Some(vec!["zh-cn", "tokens", "bitcoin", ""]));
    }
}
