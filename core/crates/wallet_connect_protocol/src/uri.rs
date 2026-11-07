use gem_encoding::percent_decode_lossy;

use crate::ProtocolError;
use crate::crypto::{Key, parse_key};
use crate::sign::method::METHOD_SESSION_AUTHENTICATE;
use crate::sign::types::RELAY_PROTOCOL;

const SCHEME: &str = "wc:";
const VERSION: &str = "2";
const TOPIC_LENGTH: usize = 64;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct PairingUri {
    pub topic: String,
    pub key: Key,
    pub expiry: Option<u64>,
    pub authenticates: bool,
}

impl PairingUri {
    pub fn parse(uri: &str, now: u64) -> Result<Self, ProtocolError> {
        let uri = percent_decode_lossy(uri.trim());
        let body = uri.strip_prefix(SCHEME).ok_or_else(|| invalid("scheme"))?;
        let body = body.strip_prefix("//").unwrap_or(body);
        let (topic, rest) = body.split_once('@').ok_or_else(|| invalid("topic"))?;
        let (version, query) = rest.split_once('?').ok_or_else(|| invalid("query"))?;
        if version != VERSION {
            return Err(invalid("version"));
        }
        if topic.len() != TOPIC_LENGTH || !topic.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid("topic"));
        }
        let parameter = |name: &str| query.split('&').find_map(|pair| pair.split_once('=').filter(|(key, _)| *key == name).map(|(_, value)| value));
        if parameter("relay-protocol") != Some(RELAY_PROTOCOL) {
            return Err(invalid("relay-protocol"));
        }
        let key = parse_key(parameter("symKey").ok_or_else(|| invalid("symKey"))?)?;
        let expiry = parameter("expiryTimestamp").map(|value| value.parse::<u64>().map_err(|_| invalid("expiryTimestamp"))).transpose()?;
        if expiry.is_some_and(|expiry| expiry <= now) {
            return Err(ProtocolError::ExpiredUri);
        }
        let authenticates = parameter("methods").is_some_and(|methods| methods.split(',').any(|method| method.trim_matches(['[', ']']) == METHOD_SESSION_AUTHENTICATE));
        Ok(Self {
            topic: topic.to_ascii_lowercase(),
            key,
            expiry,
            authenticates,
        })
    }
}

fn invalid(reason: &str) -> ProtocolError {
    ProtocolError::InvalidUri(reason.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_SYM_KEY, TEST_TOPIC, test_key};

    #[test]
    fn test_parse() {
        let uri = format!("wc:{TEST_TOPIC}@2?relay-protocol=irn&symKey={TEST_SYM_KEY}&expiryTimestamp=1700000300&methods=wc_sessionAuthenticate");

        assert_eq!(
            PairingUri::parse(&uri, 1_700_000_000).unwrap(),
            PairingUri {
                topic: TEST_TOPIC.to_string(),
                key: test_key(TEST_SYM_KEY),
                expiry: Some(1_700_000_300),
                authenticates: true,
            }
        );
        assert_eq!(PairingUri::parse(&uri, 1_700_000_300), Err(ProtocolError::ExpiredUri));

        let encoded = format!("wc%3A{TEST_TOPIC}%402%3FsymKey%3D{TEST_SYM_KEY}%26relay-protocol%3Dirn%26methods%3D%5Bwc_sessionPropose%5D%2C%5Bwc_authRequest%5D");
        let parsed = PairingUri::parse(&encoded, 1_700_000_000).unwrap();
        assert_eq!((parsed.expiry, parsed.authenticates), (None, false));
        assert_eq!(PairingUri::parse(&format!("wc://{TEST_TOPIC}@2?relay-protocol=irn&symKey={TEST_SYM_KEY}"), 0).unwrap().topic, TEST_TOPIC);
    }

    #[test]
    fn test_parse_rejects_malformed_uris() {
        for (uri, reason) in [
            (format!("https://{TEST_TOPIC}@2?relay-protocol=irn&symKey={TEST_SYM_KEY}"), "scheme"),
            (format!("wc:{TEST_TOPIC}@1?relay-protocol=irn&symKey={TEST_SYM_KEY}"), "version"),
            (format!("wc:abc@2?relay-protocol=irn&symKey={TEST_SYM_KEY}"), "topic"),
            (format!("wc:{TEST_TOPIC}?relay-protocol=irn"), "topic"),
            (format!("wc:{TEST_TOPIC}@2"), "query"),
            (format!("wc:{TEST_TOPIC}@2?relay-protocol=waku&symKey={TEST_SYM_KEY}"), "relay-protocol"),
            (format!("wc:{TEST_TOPIC}@2?relay-protocol=irn"), "symKey"),
            (format!("wc:{TEST_TOPIC}@2?relay-protocol=irn&symKey={TEST_SYM_KEY}&expiryTimestamp=soon"), "expiryTimestamp"),
        ] {
            assert_eq!(PairingUri::parse(&uri, 0), Err(ProtocolError::InvalidUri(reason.to_string())), "{uri}");
        }
        assert_eq!(PairingUri::parse(&format!("wc:{TEST_TOPIC}@2?relay-protocol=irn&symKey=abcd"), 0), Err(ProtocolError::InvalidKey));
    }
}
